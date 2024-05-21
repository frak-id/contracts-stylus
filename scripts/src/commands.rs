use alloy::{
    network::TransactionBuilder,
    primitives::{Address, U256},
    providers::Provider,
    rpc::types::eth::TransactionRequest,
    sol,
};
use alloy_primitives::address;
use tracing::info;

use crate::{
    cli::{DeployContractsArgs, InitContractsArgs},
    errors::ScriptError,
    utils::{build_stylus_contract, deploy_stylus_contract, read_deployed_addresses, RpcProvider},
};

/// Deploy the Frak consumption contracts
pub async fn deploy_contracts(
    _args: DeployContractsArgs,
    rpc_url: &str,
    priv_key: &str,
    client: RpcProvider,
) -> Result<(), ScriptError> {
    // Build the contracts
    info!("Building contracts...");
    let wasm_file_path = build_stylus_contract()?;
    info!("Built with success");

    // Deploy them
    info!("Deploying contracts...");
    deploy_stylus_contract(wasm_file_path, rpc_url, priv_key, client.clone()).await?;
    info!("Deployed with success");

    Ok(())
}

pub async fn init_contracts(
    args: InitContractsArgs,
    _rpc_url: &str,
    _priv_key: &str,
    client: RpcProvider,
) -> Result<(), ScriptError> {
    // Fetch contract address from json
    let deployed_address = read_deployed_addresses("deployed.json", "consumption")?;

    // Perform the init call
    info!("Performing init call...");

    // Parse the owner address
    let owner_address = args.owner.parse::<Address>().unwrap();

    // Build the tx
    let tx_request = TransactionRequest::default()
        .to(deployed_address)
        .with_call(&doNothingCall {})
        .with_value(U256::from(0));

    // Send it
    let pending_tx = client
        .send_transaction(tx_request)
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    println!("Pending init transaction... {}", pending_tx.tx_hash());

    // Wait for the transaction to be included.
    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    println!("Init tx done on block: {}", receipt.block_number.unwrap());

    Ok(())
}

sol! {
    function initialize(address owner) external;
    function doNothing() external;
}
