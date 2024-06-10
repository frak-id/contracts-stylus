//! Definitions of CLI arguments and commands for deploy scripts

use alloy::primitives::U256;
use clap::{Args, Parser, Subcommand};
use tracing::info;

use crate::{
    commands::{create_platform, deploy_contracts, send_test_ccu},
    errors::ScriptError,
    tx::client::RpcProvider,
};

/// Scripts for deploying & upgrading the Renegade Stylus contracts
#[derive(Parser)]
pub struct Cli {
    /// The command to run
    #[command(subcommand)]
    pub command: Command,
}

/// The possible CLI commands
#[derive(Subcommand)]
pub enum Command {
    /// Deploy all the contracts
    DeployContracts(DeployContractsArgs),
    /// Create a new platform
    CreatePlatform(CreatePlatformArgs),
    /// Send test CCU
    SendTestCcu(SendTestCcuArgs),
}

impl Command {
    /// Run the command
    pub async fn run(self, client: RpcProvider) -> Result<(), ScriptError> {
        match self {
            Command::DeployContracts(args) => {
                info!("Deploying contracts...");
                deploy_contracts(args, client).await?;
                Ok(())
            }
            Command::CreatePlatform(args) => {
                info!("Creating up platform...");
                create_platform(args, client).await?;
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

/// Deploy contracts
#[derive(Args)]
pub struct DeployContractsArgs {}

/// Setup some test platforms
#[derive(Args)]
pub struct CreatePlatformArgs {
    /// Address of the owner of the platforms
    #[arg(short, long)]
    pub owner: String,
    /// Type of the content
    #[arg(short, long)]
    pub content_type: U256,
    /// Type of the content
    #[arg(long)]
    pub origin: String,
}

/// Setup some test platforms
#[derive(Args)]
pub struct SendTestCcuArgs {
    /// Address of the user for which we will generate CCU
    #[arg(short, long)]
    pub user: String,
    /// The id of the platform
    #[arg(short, long)]
    pub platform_id: String,
}
