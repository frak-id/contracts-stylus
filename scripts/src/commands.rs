use crate::abi::initializeCall;
use crate::cli::DeployContractsArgs;
use crate::errors::ScriptError;
use crate::utils::{build_stylus_contract, deploy_stylus_contract, LocalWalletHttpClient};
use alloy_primitives::Address;
use alloy_sol_types::SolCall;
use std::sync::Arc;
use ethers::abi::AbiEncode;
use tracing::info;

/// Deploy the Frak consumption contracts
pub async fn deploy_contracts(
    args: DeployContractsArgs,
    rpc_url: &str,
    priv_key: &str,
    client: Arc<LocalWalletHttpClient>,
) -> Result<(), ScriptError> {
    // Build the contracts
    info!("Building contracts...");
    let wasm_file_path = build_stylus_contract()?;
    info!("Built with success");

    // Deploy them
    info!("Deploying contracts...");
    let address = deploy_stylus_contract(wasm_file_path, rpc_url, priv_key, client.clone()).await?;
    info!("Deployed with success");

    // Perform the init call
    info!("Performing init call...");
    // Should switch from ethers to full on alloy
    // Sending tx stuff: https://alloy-rs.github.io/alloy/alloy_signer/index.html
    let init_calldata = get_init_calldata(args)?;
    println!("Init calldata: {}", init_calldata.encode_hex());

    // TODO: What should we do here:
    //  - Build the contract
    //  - Deploy it
    //  - Call initialise function on it

    Ok(())
}

// Get the initial calldata for the contract
fn get_init_calldata(args: DeployContractsArgs) -> Result<Vec<u8>, ScriptError> {
    println!("Owner: {}", args.owner);
    let owner_address = Address::from_slice(args.owner.as_bytes());
    println!("Parsed owner: {}", owner_address);

    Ok(initializeCall {
        owner: owner_address,
    }
    .encode())
}
