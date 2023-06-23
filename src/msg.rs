use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Binary, CustomMsg, Uint256};

#[cw_serde]
pub struct InstantiateMsg {
    pub retry_delay: u64,
    pub job_id: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Swap {
        swap_id: Uint256,
        amount_out_min: Uint256,
        number_trades: Uint256,
    },
    SetPaloma {},
    UpdateCompass {
        new_compass: String,
    },
    UpdateRefundWallet {
        new_refund_wallet: String,
    },
    UpdateFee {
        fee: Uint256,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetJobIdResponse)]
    GetJobId {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetJobIdResponse {
    pub job_id: String,
}

#[cw_serde]
pub struct PalomaMsg {
    /// The ID of the paloma scheduled job to run.
    pub job_id: String,
    /// The payload, ABI encoded for the target chain.
    pub payload: Binary,
}

impl CustomMsg for PalomaMsg {}
