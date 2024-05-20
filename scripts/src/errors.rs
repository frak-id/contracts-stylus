//! Definitions of errors that can occur during the execution of the contract management scripts

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
};

/// Errors that can occur during the execution of the contract management scripts
#[derive(Debug)]
pub enum ScriptError {
    /// Error when building output file
    JsonOutputError(String),
    /// Error when creating the client
    ClientInitialization(String),
    /// Error when fetching the nonce to deploy a contract
    NonceFetching(String),
    /// Error deploying a contract
    ContractDeployment(String),
    /// Error compiling a contract
    ContractCompilation(String),
    /// Error calling a contract method
    ContractInteraction(String),
}

impl Display for ScriptError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::JsonOutputError(s) => write!(f, "error writing json output: {}", s),
            ScriptError::ClientInitialization(s) => write!(f, "error during client init: {}", s),
            ScriptError::NonceFetching(s) => {
                write!(f, "error during nonce fetching for client signing: {}", s)
            }
            ScriptError::ContractDeployment(s) => write!(f, "error deploying contract: {}", s),
            ScriptError::ContractCompilation(s) => write!(f, "error compiling contract: {}", s),
            ScriptError::ContractInteraction(s) => {
                write!(f, "error interacting with contract: {}", s)
            }
        }
    }
}

impl Error for ScriptError {}
