use crate::ContractError::AllPending;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint256,
};
use cw2::set_contract_version;
use ethabi::{Contract, Function, Param, ParamType, StateMutability, Token, Uint};
use std::collections::BTreeMap;
use std::str::FromStr;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetJobIdResponse, InstantiateMsg, PalomaMsg, QueryMsg};
use crate::state::{State, RETRY_DELAY, STATE, WITHDRAW_TIMESTAMP};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:dca-bot-pancakeswap-cw";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        job_id: msg.job_id.clone(),
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    RETRY_DELAY.save(deps.storage, &msg.retry_delay)?;
    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("job_id", msg.job_id))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<PalomaMsg>, ContractError> {
    match msg {
        ExecuteMsg::Swap {
            swap_id,
            amount_out_min,
            number_trades,
        } => swap(deps, env, swap_id, amount_out_min, number_trades),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetJobId {} => to_binary(&get_job_id(deps)?),
    }
}

fn swap(
    deps: DepsMut,
    env: Env,
    swap_id: Uint256,
    amount_out_min: Uint256,
    number_trades: Uint256,
) -> Result<Response<PalomaMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    #[allow(deprecated)]
    let contract: Contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![(
            "swap".to_string(),
            vec![Function {
                name: "swap".to_string(),
                inputs: vec![Param {
                    name: "swap_id".to_string(),
                    kind: ParamType::Uint(256),
                    internal_type: None,
                },
                Param{
                    name: "amount_out_min".to_string(),
                    kind: ParamType::Uint(256),
                    internal_type: None,
                }
                ],
                outputs: Vec::new(),
                constant: None,
                state_mutability: StateMutability::NonPayable,
            }],
        )]),
        events: BTreeMap::new(),
        errors: BTreeMap::new(),
        receive: false,
        fallback: false,
    };

    let mut tokens: Vec<Token> = vec![];
    let retry_delay: u64 = RETRY_DELAY.load(deps.storage)?;
    if let Some(timestamp) =
        WITHDRAW_TIMESTAMP.may_load(deps.storage, (swap_id.to_be_bytes().as_slice(), number_trades.to_be_bytes().as_slice()))?
    {
        if timestamp.plus_seconds(retry_delay).lt(&env.block.time) {
            tokens = vec![Token::Uint(Uint::from_str(swap_id.to_string().as_str()).unwrap()), Token::Uint(Uint::from_str(amount_out_min.to_string().as_str()).unwrap())];
            WITHDRAW_TIMESTAMP.save(
                deps.storage,
                (swap_id.to_be_bytes().as_slice(), number_trades.to_be_bytes().as_slice()),
                &env.block.time,
            )?;
        }
    } else {
        tokens = vec![Token::Uint(Uint::from_str(swap_id.to_string().as_str()).unwrap()), Token::Uint(Uint::from_str(amount_out_min.to_string().as_str()).unwrap())];
        WITHDRAW_TIMESTAMP.save(
            deps.storage,
            (swap_id.to_be_bytes().as_slice(), number_trades.to_be_bytes().as_slice()),
            &env.block.time,
        )?;
    }

    if tokens.is_empty() {
        Err(AllPending {})
    } else {
        Ok(Response::new()
            .add_message(CosmosMsg::Custom(PalomaMsg {
                job_id: state.job_id,
                payload: Binary(
                    contract
                        .function("swap")
                        .unwrap()
                        .encode_input(tokens.as_slice())
                        .unwrap(),
                ),
            }))
            .add_attribute("action", "swap"))
    }
}

fn get_job_id(deps: Deps) -> StdResult<GetJobIdResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(GetJobIdResponse {
        job_id: state.job_id,
    })
}

#[cfg(test)]
mod tests {}
