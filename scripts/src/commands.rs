
/// Deploy the Frak consumption contracts
pub async fn deploy_contracts(
    args: DeployTestContractsArgs,
    rpc_url: &str,
    priv_key: &str,
    client: Arc<LocalWalletHttpClient>,
    deployments_path: &str,
) -> Result<(), ScriptError> {
    info!("Deploying contracts...");

}