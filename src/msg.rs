use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub tot_deposit: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    deposit {}, //We could insert the amount to check if it is consistent with the fund sent to the bank module of the blockchain
    transfer {amount: Uint128, receiver: Addr},
    withdraw {amount: Uint128, receiver: Addr},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetDepositResponse)]
    GetDeposit {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetDepositResponse {
    pub Address: Addr,
    pub deposit: Uint128,
}
