//! Utilities for the deploy scripts.

use std::process::Command;

use crate::errors::ScriptError;

/// Executes a command, returning an error if the command fails
pub fn command_success_or(mut cmd: Command, err_msg: &str) -> Result<(), ScriptError> {
    if !cmd
        .output()
        .map_err(|e| ScriptError::ContractCompilation(e.to_string()))?
        .status
        .success()
    {
        Err(ScriptError::ContractCompilation(String::from(err_msg)))
    } else {
        Ok(())
    }
}
