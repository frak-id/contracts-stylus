use std::{fmt::LowerHex, fs, fs::File, io::Read, path::PathBuf};

use json::JsonValue;

use crate::errors::ScriptError;

pub enum OutputKeys {
    // Key related to a deployment
    Deployment { key: &'static str },
    // Key related to the init of a contract
    Init { key: &'static str },
    // Key related to a platform creation
    Tx { key: &'static str, tx_key: String },
}

/// Read a deployed address
pub fn read_output_file(file_path: &str, key: OutputKeys) -> Result<String, ScriptError> {
    // If the file doesn't exist, create it
    if !PathBuf::from(file_path).exists() {
        return Err(ScriptError::JsonOutputError(String::from(
            "Deployed addresses file not found",
        )));
    }

    // Parse it's json content into objects
    let parsed_json = get_json_from_file(file_path)?;
    let final_key = match key {
        OutputKeys::Deployment { key } => parsed_json[key]["deploy"].clone(),
        OutputKeys::Init { key } => parsed_json[key]["init"].clone(),
        OutputKeys::Tx { key, tx_key } => parsed_json[key]["txs"][tx_key].clone(),
    };

    Ok(final_key.as_str().unwrap().to_string())
}

/// Writes the given address for the deployed contract
pub fn write_output_file<T: LowerHex>(
    file_path: &str,
    key: OutputKeys,
    value: T,
) -> Result<(), ScriptError> {
    // If the file doesn't exist, create it
    if !PathBuf::from(file_path).exists() {
        fs::write(file_path, "{}").map_err(|e| ScriptError::JsonOutputError(e.to_string()))?;
    }

    // Parse it's json content into objects
    let mut parsed_json = get_json_from_file(file_path)?;

    // Update the right key
    match key {
        OutputKeys::Deployment { key } => {
            parsed_json[key]["deploy"] = JsonValue::String(format!("{value:#x}"))
        }
        OutputKeys::Init { key } => {
            parsed_json[key]["init"] = JsonValue::String(format!("{value:#x}"))
        }
        OutputKeys::Tx { key, tx_key } => {
            parsed_json[key]["txs"][tx_key] = JsonValue::String(format!("{value:#x}"))
        }
    };

    // Write the updated json back to the file
    fs::write(file_path, json::stringify_pretty(parsed_json, 4))
        .map_err(|e| ScriptError::JsonOutputError(e.to_string()))?;

    Ok(())
}

/// Parses the JSON file at the given path
fn get_json_from_file(file_path: &str) -> Result<JsonValue, ScriptError> {
    let mut file_contents = String::new();
    File::open(file_path)
        .map_err(|e| ScriptError::JsonOutputError(e.to_string()))?
        .read_to_string(&mut file_contents)
        .map_err(|e| ScriptError::JsonOutputError(e.to_string()))?;

    json::parse(&file_contents).map_err(|e| ScriptError::JsonOutputError(e.to_string()))
}
