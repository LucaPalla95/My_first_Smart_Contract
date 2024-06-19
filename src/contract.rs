#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Addr, Uint128};
use cw2::set_contract_version;
use crate::error::ContractError;
use crate::msg::{ExecuteMsg, GetDepositResponse, InstantiateMsg, QueryMsg, GetAllDepositResponse, GetTotalDepositResponse, GetStateResponse};
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
        ExecuteMsg::Deposit {} => execute::deposit_fund(deps, info),
        ExecuteMsg::Transfer {amount, receiver } => execute::transfer_fund(deps, info, amount, receiver),
        ExecuteMsg::Withdraw {amount } => execute::withdraw_fund(deps, info, amount),
    }
}

pub mod execute {
    use cosmwasm_std::{BankMsg, Coin};

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
            return Err(ContractError::InvalidDepositAmount{});
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
            return Err(ContractError::NoEmptyFunds {});
        }

        let receiver = deps.api.addr_validate(&receiver.to_string())?;

        if amount.is_zero() {
            return Err(ContractError::InvalidTransferAmount{});
        }

        // check if deposits are sufficient
        BALANCES.update(deps.storage, info.sender.clone(), |balance: Option<Uint128>| {
            if let Some(balance_sender) = balance{
                if balance_sender >= amount {
                    Ok::<Uint128,ContractError>(balance_sender - amount)
                } else {
                    Err(ContractError::TransferFundsExceedsBalance {  })
                }
            } else {
                Err(ContractError::AddressHasNotDeposit {  })
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

        // check no funds are sent
        if !info.funds.is_empty() {
            return Err(ContractError::NoEmptyFunds {});
        }

        // Upload confuguration and balance
        let config = CONFIG.load(deps.storage)?;

        // Find the amount of allowed_denom sent with the transaction
        if amount.is_zero() {
            return Err(ContractError::InvalidDepositAmount {});
        }

        let receiver = info.sender.clone();
        
        // Update balance if sufficient amount was deposited.
        BALANCES.update(deps.storage, info.sender.clone(), |balance: Option<Uint128>| {
            if let Some(balance_sender) = balance{
                if balance_sender >= amount {
                    Ok::<Uint128,ContractError>(balance_sender - amount)
                } else {
                    Err(ContractError::WithdrawFundsExceedsBalance {  })
                }
            } else {
                Err(ContractError::AddressHasNotDeposit {  })
            }
        })?;
        Ok(Response::new().add_attribute("action", "withdraw").add_message(BankMsg::Send{ 
            amount: vec![Coin::new(amount, config.allowed_denom.clone())], 
            to_address: receiver.to_string(),
        }).add_attribute("amount", amount)
        .add_attribute("receiver", receiver))
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(
    deps: Deps,
    _env: Env, 
    msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetState {} => to_json_binary(&query::state(deps)?),
        QueryMsg::GetDeposit {owner} => to_json_binary(&query::deposit(deps, owner)?),
        QueryMsg::GetAllDeposit {} => to_json_binary(&query::all_deposits(deps)?),
        QueryMsg::GetTotalDeposit {} => to_json_binary(&query::totaldeposit(deps)?),
        }
    }

pub mod query {

    use cosmwasm_std::Order;

    use super::*;

    pub fn state(
        deps: Deps) -> StdResult<GetStateResponse> {
        let config = CONFIG.load(deps.storage)?;
        Ok(GetStateResponse {allowed_denom: config.allowed_denom})
    }

    pub fn deposit(
        deps: Deps, 
        owner: Addr) -> StdResult<GetDepositResponse> {
        let balance = BALANCES.load(deps.storage, owner.clone())?;
        Ok(GetDepositResponse {address: owner, deposit: balance })
    }

    pub fn all_deposits(deps: Deps) -> StdResult<Vec<GetAllDepositResponse>> {
        // 
        let balances = BALANCES.range(deps.storage, None, None, Order::Ascending); // LOAD MEGLIO
    
        let response = balances
            .map(|item| {
                let (address, balance) = item?;
                Ok(GetAllDepositResponse {
                    address: address,
                    totaldeposit: balance,
                })
            })
            .collect::<StdResult<Vec<_>>>()?;
    
        Ok(response)
    }

    pub fn totaldeposit(deps: Deps) -> StdResult<GetTotalDepositResponse> {
        // Iterazione sui bilanci in ordine ascendente
        let balances = BALANCES.range(deps.storage, None, None, Order::Ascending);
    
        // Calcolo del totale dei depositi con gestione degli errori
        let total_deposit = balances
            .fold(Uint128::zero(), |acc, item| {
                let (_, balance) = item.expect("Item should be there");
                acc + balance
            });
    
        Ok(GetTotalDepositResponse { totaldeposit: total_deposit })
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_json};


    // Istantiate
    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &[]);

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // query the state
        let res_q = query(deps.as_ref(), mock_env(), QueryMsg::GetState {}).unwrap();
        let value: GetStateResponse = from_json(&res_q).unwrap();
        assert_eq!("tsy", value.allowed_denom);
    }

    // Test deposit successful
    #[test]
    fn test_deposit_successfully() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        // Check the balance in the storage
        let balance = BALANCES.load(&deps.storage, Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0")).unwrap();
        assert_eq!(balance, Uint128::new(1000));
    
        // Check if the correct event is emitted
        // let events = res.events;
        // assert_eq!(events.len(), 1);
        // assert_eq!(events[0].ty, "deposit");
        // assert_eq!(events[0].attributes.len(), 2);
        // assert_eq!(events[0].attributes[0].key, "sender");
        // assert_eq!(events[0].attributes[0].value, "cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0");
        // assert_eq!(events[0].attributes[1].key, "amount");
        // assert_eq!(events[0].attributes[1].value, "2tsy");
    }

    // Test deposit error 0
    #[test]
    fn test_deposit_0_deposit_error() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(0, "tsy".to_string()));
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::InvalidDepositAmount {}) => {}
            _ => panic!("Must return Invalid Deposit Amount error"),
        }
    }
    // Test deposit error 0 parte 2
    #[test]
    fn test_deposit_0_deposit_error_2() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "thi".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::InvalidDepositAmount {}) => {}
            _ => panic!("Must return Invalid Deposit Amount error"),
        }
    }

    // Test transfer successfully
    #[test]
    fn test_transfer_successfully() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Transfer {receiver: Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx1"), amount: Uint128::new(2),};
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let res_q = query(deps.as_ref(), mock_env(), QueryMsg::GetDeposit { owner: Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx1") }).unwrap();
        let value: GetDepositResponse = from_json(&res_q).unwrap();
        assert_eq!(value.deposit, Uint128::new(2));

    }

    // Test transfer error fund not empty
    #[test]
    fn test_transfer_error_fund_not_empty() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Transfer {receiver: Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx1"), amount: Uint128::new(2),};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::NoEmptyFunds {}) => {}
            _ => panic!("Must return Fund Not Empty error"),
        }
    }

    // Test transfer error invalid address
    #[test]
    fn test_transfer_error_invalid_address() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Transfer {receiver: Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx1"), amount: Uint128::new(2)};
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            _ => panic!("Must return Invalid Address error"),
        }
    }

    // Test transfer error 0
    #[test]
    fn test_transfer_error_0_transfer() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Transfer {receiver: Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx1"), amount: Uint128::new(0)};
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::InvalidTransferAmount {}) => {}
            _ => panic!("Must return Invalid Transfer Amount error"),
        }
    }

    // Test transfer error sender no deposit
    #[test]
    fn test_transfer_error_sender_no_deposit() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Transfer {receiver: Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx1"), amount: Uint128::new(2),};
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx1"), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::AddressHasNotDeposit {}) => {}
            _ => panic!("Must return Address Has No Deposit error"),
        }
    }

    // Test transfer error transfer exceed deposit
    #[test]
    fn test_transfer_error_transfer_exceed_deposit() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Transfer {receiver: Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx1"), amount: Uint128::new(10000),};
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::TransferFundsExceedsBalance {}) => {}
            _ => panic!("Must return Transfer Exceed Balance error"),
        }
    }
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
    // Test withdraw successfully
    #[test]
    fn test_withdraw_successfully() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Withdraw { amount: Uint128::new(2)};
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(res.messages.len(), 0);

        let res_q = query(deps.as_ref(), mock_env(), QueryMsg::GetDeposit { owner: Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0") }).unwrap();
        let value: GetDepositResponse = from_json(&res_q).unwrap();
        assert_eq!(value.deposit, Uint128::new(998));
    }

    // Test withdraw error fund not empty
    #[test]
    fn test_withdraw_error_fund_not_empty() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Withdraw {amount: Uint128::new(2)};
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::NoEmptyFunds {}) => {}
            _ => panic!("Must return Fund Not Empty error"),
        }
    }

    // Test withdraw error 0
    #[test]
    fn test_withdraw_error_0_withdraw() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Withdraw {amount: Uint128::new(0)};
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::InvalidWithdrawAmount {}) => {}
            _ => panic!("Must return Invalid Withdraw Amount error"),
        }
    }

    // Test withdraw error receiver no deposit
    #[test]
    fn test_withdraw_error_sender_no_deposit() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = ExecuteMsg::Withdraw {amount: Uint128::new(2)};
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx2"), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::AddressHasNotDeposit {}) => {}
            _ => panic!("Must return Address Has No Deposit error"),
        }
    }

    // Test withdraw error withdraw exceed deposit
    #[test]
    fn test_withdraw_error_withdraw_exceed_deposit() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg { allowed_denom: "tsy".to_string() };
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &coins(1000, "tsy".to_string()));
        let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Deposit{};
        let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

        let msg = ExecuteMsg::Withdraw {amount: Uint128::new(10000)};
        let info = message_info(&Addr::unchecked("cosmos1xv9tklw7d82sezh9ha4c6w7422k3halglxxxx0"), &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::WithdrawFundsExceedsBalance {}) => {}
            _ => panic!("Must return Withdraw Exceed Balance error"),
        }
    }
}
