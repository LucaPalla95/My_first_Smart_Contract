use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Amount deposit must be greater than 0")]
    InvalidDepositAmount(),

    #[error("Funds field should not be empty")]
    EmptyFunds(),

    #[error("Balance is lower than amount to transfer")]
    TransferFundsExceedsBalance(),

    #[error("The address selected does not have a deposit")]
    AddressHasNotDeposit(),

    #[error("Balance is lower than amount to withdraw")]
    WithdrawFundsExceedsBalance(),
}
