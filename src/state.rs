use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub tot_deposit: Uint128,
}

pub const STATE: Item<State> = Item::new("state");
pub const BALANCES: Map<Addr, Uint128> = Map::new("balances");
