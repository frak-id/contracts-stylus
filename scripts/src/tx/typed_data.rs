use std::env;

use alloy::{
    core::sol,
    primitives::{keccak256, Address, B256, U256},
    providers::Provider,
    signers::{k256::ecdsa::SigningKey, wallet::Wallet, Signature, Signer},
    sol_types::{eip712_domain, Eip712Domain},
};

use crate::{errors::ScriptError, tx::client::RpcProvider};

/// Get the current domain separator
pub async fn get_domain(deployed_address: Address, client: RpcProvider) -> Eip712Domain {
    // Fetch the chain id
    let chain_id = client.get_chain_id().await.unwrap();

    eip712_domain! {
        name: "ContentConsumption",
        version: "0.0.1",
        chain_id: chain_id,
        verifying_contract: deployed_address,
    }
}

/// Get the validate consumption signature
pub async fn get_validate_consumption_signature(
    deployed_address: Address,
    client: RpcProvider,
    user: Address,
    platform_id: B256,
    added_consumption: U256,
    nonce: U256,
    deadline: U256,
) -> Result<Signature, ScriptError> {
    // Build the validate consumption struct hash
    sol! {
        struct ValidateConsumption {
            address user;
            bytes32 platformId;
            uint256 addedConsumption;
            uint256 nonce;
            uint256 deadline;
        }
    }
    let validate_consumption = ValidateConsumption {
        user,
        platformId: platform_id,
        addedConsumption: added_consumption,
        nonce,
        deadline,
    };

    // Get our signer
    let signer = env::var("PRIVATE_KEY")
        .unwrap()
        .parse::<Wallet<SigningKey>>()
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    // Get the domain
    let domain = get_domain(deployed_address, client.clone()).await;

    // Ask the client to sign the given typed data
    Ok(signer
        .sign_typed_data(&validate_consumption, &domain)
        .await
        .unwrap())
}

/// Get the validate consumption signature
pub async fn get_register_platform_signature(
    deployed_address: Address,
    client: RpcProvider,
    owner: Address,
    name: String,
    origin: String,
    deadline: U256,
) -> Result<Signature, ScriptError> {
    // Build the validate consumption struct hash
    sol! {
        struct CreateNewPlatform {
            address owner;
            bytes32 name;
            bytes32 origin;
            uint256 deadline;
        }
    }
    let create_platform = CreateNewPlatform {
        owner,
        name: keccak256(&name.as_bytes()),
        origin: keccak256(&origin.as_bytes()),
        deadline,
    };

    // Get our signer
    let signer = env::var("PRIVATE_KEY")
        .unwrap()
        .parse::<Wallet<SigningKey>>()
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    // Get the domain
    let domain = get_domain(deployed_address, client.clone()).await;

    // Ask the client to sign the given typed data
    Ok(signer
        .sign_typed_data(&create_platform, &domain)
        .await
        .unwrap())
}
