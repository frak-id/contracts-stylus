use clap::Parser;
use scripts::constants::DEFAULT_RPC;
use scripts::utils::setup_client;
use scripts::{cli::Cli, errors::ScriptError};

#[tokio::main]
async fn main() -> Result<(), ScriptError> {
    let Cli {
        priv_key,
        rpc_url,
        command,
    } = Cli::parse();

    tracing_subscriber::fmt().pretty().init();

    // Rpc url, if not present, default to "https://stylusv2.arbitrum.io/rpc
    let rpc_url = rpc_url.unwrap_or_else(|| DEFAULT_RPC.to_string());

    // Build our RPC client with signer
    let client = setup_client(&priv_key, &rpc_url).await?;

    command.run(client, &rpc_url, &priv_key).await
}
