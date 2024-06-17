#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Uint128};
use cw2::set_contract_version;
use cw_storage_plus::{Item, Map};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetDepositResponse, InstantiateMsg, QueryMsg};
use crate::state::{CONFIG, Config, BALANCES};

// version info for migration info hh
const CONTRACT_NAME: &str = "crates.io:my_first_contract";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env, 
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config = Config {
        allowed_denom: msg.allowed_denom,
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("allowed denom", &config.allowed_denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::deposit {} => execute::deposit_fund(deps, info),
        ExecuteMsg::transfer {amount, receiver } => execute::transfer_fund(deps, info, amount, receiver),
        ExecuteMsg::withdraw {amount } => execute::withdraw_fund(deps, info, amount),
    }
}

pub mod execute {
    use cosmwasm_std::BankMsg;

    use crate::state::BALANCES;

    use super::*;

    pub fn deposit_fund(
        deps: DepsMut, 
        info: MessageInfo,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;
    
        // Find the amount of allowed_denom sent with the transaction
        let amount = info
            .funds // funds sent
            .iter() // iterate over coins sent
            // TODO: Check if the && is correct
            .find(|c| c.denom == config.allowed_denom) // search for the coin which is allowed by the contract
            .map(|c| c.amount) // get the amount
            .unwrap_or_else(Uint128::zero); // if not found return 0
    
        if amount.is_zero() {
            //TODO: Change the error message
            return Err(ContractError::Unauthorized {});
        }
    
        let depositor = info.sender;
        BALANCES.update(deps.storage, depositor.clone(), |balance: Option<Uint128>| {
            Ok::<Uint128,ContractError>(balance.unwrap_or_else(Uint128::zero) + amount)
        })?;
    
        Ok(Response::new()
            .add_attribute("action", "deposit_funds")
            .add_attribute("depositor", depositor)
            .add_attribute("amount", amount))
    }

    pub fn transfer_fund(
        deps: DepsMut,
        info: MessageInfo, 
        amount: Uint128,
        receiver: Addr,
    ) -> Result<Response, ContractError> {

        // check no funds are sent
        if !info.funds.is_empty() {
            return Err(ContractError::Unauthorized {});
        }

        let receiver = deps.api.addr_validate(&receiver.to_string())?;

        if amount.is_zero() {
            return Err(ContractError::Unauthorized {});
        }

        // check if deposits are sufficient
        BALANCES.update(deps.storage, info.sender.clone(), |balance: Option<Uint128>| {
            if let Some(balance_sender) = balance{
                if balance_sender >= amount {
                    Ok::<Uint128,ContractError>(balance_sender - amount)
                } else {
                    Err(ContractError::Unauthorized {  })
                }
            } else {
                Err(ContractError::Unauthorized {  })
            }
        })?;
        // TODO: Check if the error above block also this update
        BALANCES.update(deps.storage, receiver.clone(), |balance: Option<Uint128>| {
            Ok::<Uint128,ContractError>(balance.unwrap_or_else(Uint128::zero) + amount)
        })?;

        Ok(Response::new()
        .add_attribute("action", "trasfer_fund")
        .add_attribute("sender", info.sender)
        .add_attribute("receiver", receiver)
        .add_attribute("amount", amount.to_string()))
    }

    pub fn withdraw_fund(
        deps: DepsMut,
        info: MessageInfo, 
        amount: Uint128,
    ) -> Result<Response, ContractError> {

        let receiver = deps.api.addr_validate(&receiver)?;

        // Upload confuguration and balance
        let config = CONFIG.load(deps.storage)?;
        let balance_sender: Uint128 = BALANCES.may_load(deps.storage, &info.sender)?
        ;
        Ok(Response::new().add_attribute("action", "reset").add_message(BankMsg::Send{ 
            amount: amount,
            to_address: receiver.to_string(),
        }))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_json_binary(&query::count(deps)?),
    }
}

pub mod query {
    use super::*;

    pub fn count(deps: Deps) -> StdResult<GetCountResponse> {
        let state = STATE.load(deps.storage)?;
        Ok(GetCountResponse { count: state.count })
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
//     use cosmwasm_std::{coins, from_json};

    // #[test]
    // fn proper_initialization() {
    //     let mut deps = mock_dependencies();

    //     let msg = InstantiateMsg { count: 17 };
    //     let info = mock_info("creator", &coins(1000, "earth"));

    //     // we can just call .unwrap() to assert this was a success
    //     let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    //     assert_eq!(0, res.messages.len());

    //     // it worked, let's query the state
    //     let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
    //     let value: GetCountResponse = from_json(&res).unwrap();
    //     assert_eq!(17, value.count);
    // }

//     #[test]
//     fn increment() {
//         let mut deps = mock_dependencies();

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Increment {};
//         let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // should increase counter by 1
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: GetCountResponse = from_json(&res).unwrap();
//         assert_eq!(18, value.count);
//     }

//     #[test]
//     fn reset() {
//         let mut deps = mock_dependencies();

//         let msg = InstantiateMsg { count: 17 };
//         let info = mock_info("creator", &coins(2, "token"));
//         let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

//         // beneficiary can release it
//         let unauth_info = mock_info("anyone", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
//         match res {
//             Err(ContractError::Unauthorized {}) => {}
//             _ => panic!("Must return unauthorized error"),
//         }

//         // only the original creator can reset the counter
//         let auth_info = mock_info("creator", &coins(2, "token"));
//         let msg = ExecuteMsg::Reset { count: 5 };
//         let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

//         // should now be 5
//         let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
//         let value: GetCountResponse = from_json(&res).unwrap();
//         assert_eq!(5, value.count);
//     }
// }
