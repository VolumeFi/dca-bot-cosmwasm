use crate::ContractError::{AllPending, Unauthorized};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint256,
};
use cw2::set_contract_version;
use ethabi::{Address, Contract, Function, Param, ParamType, StateMutability, Token, Uint};
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
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response<PalomaMsg>, ContractError> {
    match msg {
        ExecuteMsg::Swap {
            swap_id,
            amount_out_min,
            number_trades,
        } => swap(deps, env, swap_id, amount_out_min, number_trades),
        ExecuteMsg::SetPaloma {} => set_paloma(deps, info),
        ExecuteMsg::UpdateCompass { new_compass } => update_compass(deps, info, new_compass),
        ExecuteMsg::UpdateRefundWallet { new_refund_wallet } => {
            update_refund_wallet(deps, info, new_refund_wallet)
        }
        ExecuteMsg::UpdateFee { fee } => update_fee(deps, info, fee),
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
                inputs: vec![
                    Param {
                        name: "swap_id".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    },
                    Param {
                        name: "amount_out_min".to_string(),
                        kind: ParamType::Uint(256),
                        internal_type: None,
                    },
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
    if let Some(timestamp) = WITHDRAW_TIMESTAMP.may_load(
        deps.storage,
        (
            swap_id.to_be_bytes().as_slice(),
            number_trades.to_be_bytes().as_slice(),
        ),
    )? {
        if timestamp.plus_seconds(retry_delay).lt(&env.block.time) {
            tokens = vec![
                Token::Uint(Uint::from_big_endian(&swap_id.to_be_bytes())),
                Token::Uint(Uint::from_big_endian(&amount_out_min.to_be_bytes())),
            ];
            WITHDRAW_TIMESTAMP.save(
                deps.storage,
                (
                    swap_id.to_be_bytes().as_slice(),
                    number_trades.to_be_bytes().as_slice(),
                ),
                &env.block.time,
            )?;
        }
    } else {
        tokens = vec![
            Token::Uint(Uint::from_big_endian(&swap_id.to_be_bytes())),
            Token::Uint(Uint::from_big_endian(&amount_out_min.to_be_bytes())),
        ];
        WITHDRAW_TIMESTAMP.save(
            deps.storage,
            (
                swap_id.to_be_bytes().as_slice(),
                number_trades.to_be_bytes().as_slice(),
            ),
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

fn set_paloma(deps: DepsMut, info: MessageInfo) -> Result<Response<PalomaMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if state.owner != info.sender {
        return Err(Unauthorized {});
    }
    #[allow(deprecated)]
    let contract: Contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![(
            "set_paloma".to_string(),
            vec![Function {
                name: "set_paloma".to_string(),
                inputs: vec![],
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
    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: state.job_id,
            payload: Binary(
                contract
                    .function("set_paloma")
                    .unwrap()
                    .encode_input(&[])
                    .unwrap(),
            ),
        }))
        .add_attribute("action", "set_paloma"))
}

fn update_compass(
    deps: DepsMut,
    info: MessageInfo,
    new_compass: String,
) -> Result<Response<PalomaMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if state.owner != info.sender {
        return Err(Unauthorized {});
    }
    let new_compass_address: Address = Address::from_str(new_compass.as_str()).unwrap();
    #[allow(deprecated)]
    let contract: Contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![(
            "update_compass".to_string(),
            vec![Function {
                name: "update_compass".to_string(),
                inputs: vec![Param {
                    name: "new_compass".to_string(),
                    kind: ParamType::Address,
                    internal_type: None,
                }],
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

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: state.job_id,
            payload: Binary(
                contract
                    .function("update_compass")
                    .unwrap()
                    .encode_input(&[Token::Address(new_compass_address)])
                    .unwrap(),
            ),
        }))
        .add_attribute("action", "update_compass"))
}

fn update_refund_wallet(
    deps: DepsMut,
    info: MessageInfo,
    new_refund_wallet: String,
) -> Result<Response<PalomaMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if state.owner != info.sender {
        return Err(Unauthorized {});
    }
    let new_refund_wallet_address: Address = Address::from_str(new_refund_wallet.as_str()).unwrap();
    #[allow(deprecated)]
    let contract: Contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![(
            "update_refund_wallet".to_string(),
            vec![Function {
                name: "update_refund_wallet".to_string(),
                inputs: vec![Param {
                    name: "new_refund_wallet".to_string(),
                    kind: ParamType::Address,
                    internal_type: None,
                }],
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

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: state.job_id,
            payload: Binary(
                contract
                    .function("update_refund_wallet")
                    .unwrap()
                    .encode_input(&[Token::Address(new_refund_wallet_address)])
                    .unwrap(),
            ),
        }))
        .add_attribute("action", "update_refund_wallet"))
}

fn update_fee(
    deps: DepsMut,
    info: MessageInfo,
    fee: Uint256,
) -> Result<Response<PalomaMsg>, ContractError> {
    let state = STATE.load(deps.storage)?;
    if state.owner != info.sender {
        return Err(Unauthorized {});
    }
    #[allow(deprecated)]
    let contract: Contract = Contract {
        constructor: None,
        functions: BTreeMap::from_iter(vec![(
            "update_fee".to_string(),
            vec![Function {
                name: "update_fee".to_string(),
                inputs: vec![Param {
                    name: "new_fee".to_string(),
                    kind: ParamType::Uint(256),
                    internal_type: None,
                }],
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

    Ok(Response::new()
        .add_message(CosmosMsg::Custom(PalomaMsg {
            job_id: state.job_id,
            payload: Binary(
                contract
                    .function("update_fee")
                    .unwrap()
                    .encode_input(&[Token::Uint(Uint::from_big_endian(&fee.to_be_bytes()))])
                    .unwrap(),
            ),
        }))
        .add_attribute("action", "update_fee"))
}

fn get_job_id(deps: Deps) -> StdResult<GetJobIdResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(GetJobIdResponse {
        job_id: state.job_id,
    })
}

#[cfg(test)]
mod tests {}
