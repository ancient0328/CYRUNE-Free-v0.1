#![forbid(unsafe_code)]

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ContractError {
    #[error("field must not be empty: {field}")]
    EmptyField { field: &'static str },
    #[error("invalid identifier for {field}: {value}")]
    InvalidIdentifier { field: &'static str, value: String },
    #[error("invalid path label: {value}")]
    InvalidPathLabel { value: String },
    #[error("invalid request: {message}")]
    InvalidRequest { message: String },
}
