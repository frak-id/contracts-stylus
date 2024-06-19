//! Definitions of CLI arguments and commands for deploy scripts

use clap::{Args, Parser, Subcommand};
use tracing::info;

use crate::{
    commands::{deploy_contracts, send_test_ccu},
    errors::ScriptError,
    tx::client::RpcProvider,
};

/// Scripts for deploying & upgrading the Renegade Stylus contracts-stylus
#[derive(Parser)]
pub struct Cli {
    /// The command to run
    #[command(subcommand)]
    pub command: Command,
}

/// The possible CLI commands
#[derive(Subcommand)]
pub enum Command {
    /// Deploy all the contracts-stylus
    DeployContracts(DeployContractsArgs),
    /// Send test CCU
    SendTestCcu(SendTestCcuArgs),
}

impl Command {
    /// Run the command
    pub async fn run(self, client: RpcProvider) -> Result<(), ScriptError> {
        match self {
            Command::DeployContracts(args) => {
                info!("Deploying contracts-stylus...");
                deploy_contracts(args, client).await?;
                Ok(())
            }
            Command::SendTestCcu(args) => {
                info!("Sending test ccu...");
                send_test_ccu(args, client).await?;
                Ok(())
            }
        }
    }
}

/// Deploy contracts-stylus
#[derive(Args)]
pub struct DeployContractsArgs {
    /// The address of the content registry
    #[arg(short, long)]
    pub content_registry: String,
    /// The frak content id
    #[arg(short, long)]
    pub frak_id: String,
}

/// Setup some test platforms
#[derive(Args)]
pub struct SendTestCcuArgs {
    /// The id of the channel
    #[arg(short, long)]
    pub channel_id: String,
}
