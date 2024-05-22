use clap::Parser;
use dotenv::dotenv;
use scripts::{cli::Cli, errors::ScriptError, tx::client::create_rpc_provider};

#[tokio::main]
async fn main() -> Result<(), ScriptError> {
    // Load .env file
    dotenv().ok();

    let Cli { command } = Cli::parse();

    tracing_subscriber::fmt().pretty().init();

    // Build our RPC client with signer
    let client = create_rpc_provider().await?;

    command.run(client).await
}
