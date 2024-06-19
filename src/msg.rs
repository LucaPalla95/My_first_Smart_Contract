use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub allowed_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    deposit {}, //We could insert the amount to check if it is consistent with the fund sent to the bank module of the blockchain
    transfer {amount: Uint128, receiver: Addr},
    withdraw {amount: Uint128},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetStateResponse)]
    GetState {},
    #[returns(GetDepositResponse)]
    GetDeposit {owner: Addr},
    #[returns(GetAllDepositResponse)]
    GetAllDeposit {},
    #[returns(GetTotalDepositResponse)]
    GetTotalDeposit {},
}

// We define a custom struct for each query respons
#[cw_serde]
pub struct GetStateResponse {
    pub allowed_denom: String,
}
#[cw_serde]
pub struct GetDepositResponse {
    pub address: Addr,
    pub deposit: Uint128,
}
#[cw_serde]
pub struct GetAllDepositResponse {
    pub address: Addr,
    pub totaldeposit: Uint128,
}

#[cw_serde]
pub struct GetTotalDepositResponse {
    pub totaldeposit: Uint128,
}

