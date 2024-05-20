//! Definitions of CLI arguments and commands for deploy scripts

use crate::errors::ScriptError;
use crate::utils::LocalWalletHttpClient;
use clap::{Args, Parser, Subcommand};
use std::sync::Arc;
use tracing::info;

/// Scripts for deploying & upgrading the Renegade Stylus contracts
#[derive(Parser)]
pub struct Cli {
    /// Private key of the deployer
    #[arg(short, long)]
    pub priv_key: String,

    /// Network RPC URL
    #[arg(short, long)]
    pub rpc_url: Option<String>,

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
}

impl Command {
    /// Run the command
    pub async fn run(
        self,
        _client: Arc<LocalWalletHttpClient>,
        _rpc_url: &str,
        _priv_key: &str,
    ) -> Result<(), ScriptError> {
        match self {
            Command::DeployContracts(_args) => {
                info!("Deploying contracts...");
                // TODO: Do some shit here
                Ok(())
            }
            Command::CreatePlatform(_args) => {
                info!("Setting up platform...");
                // TODO: Do some shit here
                Ok(())
            }
        }
    }
}

/// Deploy contracts
#[derive(Args)]
pub struct DeployContractsArgs {
    /// Address of the owner of the contracts
    #[arg(short, long)]
    pub owner: String,
}

/// Setup some test platforms
#[derive(Args)]
pub struct CreatePlatformArgs {
    /// Address of the owner of the platforms
    #[arg(short, long)]
    pub owner: String,
    /// Type of the content
    #[arg(long)]
    pub content_type: u32,
    /// Type of the content
    #[arg(short, long)]
    pub origin: String,
}