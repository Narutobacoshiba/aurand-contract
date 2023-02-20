use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized{},

    #[error("UnauthorizedReceive")]
    UnauthorizedReceive{},

    #[error("InvalidAddress")]
    InvalidAddress{},

    #[error("InvalidRandomness")]
    InvalidRandomness{},

    #[error("InvalidProxyAddress")]
    InvalidProxyAddress{},
    
    #[error("AddressAlreadyRegistered")]
    AddressAlreadyRegistered{},

    #[error("UnregisteredAddress")]
    UnregisteredAddress{},

    #[error("ToManyAction")]
    ToManyAction{},

    #[error("RSAVerificationFail")]
    RSAVerificationFail{},

    #[error("Uint128Overflow")]
    Uint128Overflow{},

    #[error("InvalidApiKey")]
    InvalidApiKey{},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}