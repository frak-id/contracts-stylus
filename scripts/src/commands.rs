use crate::cli::DeployContractsArgs;
use crate::errors::ScriptError;
use crate::utils::{build_stylus_contract, deploy_stylus_contract, LocalWalletHttpClient};
use std::sync::Arc;
use tracing::info;

/// Deploy the Frak consumption contracts
pub async fn deploy_contracts(
    _args: DeployContractsArgs,
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
    let address = deploy_stylus_contract(wasm_file_path, rpc_url, priv_key, client).await?;
    info!("Deployed with success at address: {}", address);

    // TODO: What should we do here:
    //  - Build the contract
    //  - Deploy it
    //  - Call initialise function on it

    Ok(())
}
