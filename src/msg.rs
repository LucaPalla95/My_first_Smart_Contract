use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub tot_deposit: Uint128,
}

#[cw_serde]
pub enum ExecuteMsg {
    burn {amount: Uint128},
    transfer {from: Addr, to: Addr, amount: Uint128},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(GetCountResponse)]
    GetBalances {},
}

// We define a custom struct for each query response
#[cw_serde]
pub struct GetCountResponse {
    pub Address: Addr,
    pub balance: Uint128,
}
