use std::env;

use alloy::{
    hex,
    network::{Ethereum, EthereumWallet},
    primitives::B256,
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
        Identity, Provider, ProviderBuilder, ReqwestProvider,
    },
    signers::local::PrivateKeySigner,
};
use reqwest::{Client, Url};
use tracing::info;

use crate::errors::ScriptError;

/// Re-export from alloy recommend filter
type RecommendFiller =
    JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>;

/// An Ethers provider that uses a `LocalWallet` to generate signatures
/// & interfaces with the RPC endpoint over HTTP
pub type RpcProvider = FillProvider<
    JoinFill<RecommendFiller, WalletFiller<EthereumWallet>>,
    ReqwestProvider,
    alloy::transports::http::Http<Client>,
    Ethereum,
>;

/// Sets up the address and client with which to instantiate a contract for testing,
/// reading in the private key, RPC url, and contract address from the environment.
pub async fn create_rpc_provider() -> Result<RpcProvider, ScriptError> {
    // Find our private key and map it to a B256B256::from_slice(&hex::decode(privateKey).unwrap())
    let private_key = B256::from_slice(
        &hex::decode(
            env::var("PRIVATE_KEY")
                .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?,
        )
        .unwrap(),
    );
    // Create our signer
    let signer = PrivateKeySigner::from_bytes(&private_key)
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    let wallet = EthereumWallet::from(signer);

    // Create our provider with the rpc client + signer
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        .on_http(env::var("RPC_URL").unwrap().parse::<Url>().unwrap());

    // Fetch chain id
    let chain_id = provider
        .get_chain_id()
        .await
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    info!("Build client on chain ID: {}", chain_id);

    Ok(provider)
}
