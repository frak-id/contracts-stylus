use alloy::{
    network::{Ethereum, EthereumSigner},
    providers::{
        fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, SignerFiller},
        Identity, Provider, ProviderBuilder, ReqwestProvider,
    },
    signers::{k256::ecdsa::SigningKey, wallet::Wallet},
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
    JoinFill<RecommendFiller, SignerFiller<EthereumSigner>>,
    ReqwestProvider,
    alloy::transports::http::Http<Client>,
    Ethereum,
>;

/// Sets up the address and client with which to instantiate a contract for testing,
/// reading in the private key, RPC url, and contract address from the environment.
pub async fn create_rpc_provider(
    priv_key: &str,
    rpc_url: &str,
) -> Result<RpcProvider, ScriptError> {
    // Create our signer
    let signer = priv_key
        .parse::<Wallet<SigningKey>>()
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    // Create our provider with the rpc client + signer
    let provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(EthereumSigner::from(signer))
        .on_http(rpc_url.parse::<Url>().unwrap());

    // Fetch chain id
    let chain_id = provider
        .get_chain_id()
        .await
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    info!("Build client on chain ID: {}", chain_id);

    Ok(provider)
}
