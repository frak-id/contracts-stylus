use std::env;

use alloy::{
    core::sol,
    primitives::{keccak256, Address, B256, U256},
    providers::Provider,
    signers::{k256::ecdsa::SigningKey, wallet::Wallet, Signature, Signer},
    sol_types::{eip712_domain, Eip712Domain},
};

use crate::{errors::ScriptError, tx::client::RpcProvider};

/// Our global typed data signer, it will hold a few static stuff
pub struct TypedDataSigner {
    // Our signer
    signer: Wallet<SigningKey>,
    // The address of the deployed contract
    deployed_address: Address,
    // The current chain id
    chain_id: u64,
}

impl TypedDataSigner {
    // Build a new typed data signer
    pub async fn new(
        client: RpcProvider,
        deployed_address: Address,
    ) -> Result<TypedDataSigner, ScriptError> {
        // Get our signer
        let signer = env::var("PRIVATE_KEY")
            .unwrap()
            .parse::<Wallet<SigningKey>>()
            .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

        // Get the chain id
        let chain_id = client.get_chain_id().await.unwrap();

        Ok(TypedDataSigner {
            signer,
            deployed_address,
            chain_id,
        })
    }

    // Get the current domain
    pub fn get_domain(&self) -> Eip712Domain {
        eip712_domain! {
            name: "ContentConsumption",
            version: "0.0.1",
            chain_id: self.chain_id,
            verifying_contract: self.deployed_address,
        }
    }

    // Get the validate consumption signature
    pub async fn get_validate_consumption_signature(
        &self,
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

        // Ask the client to sign the given typed data
        Ok(self
            .signer
            .sign_typed_data(&validate_consumption, &self.get_domain())
            .await
            .unwrap())
    }

    /// Get the register platform signature
    pub async fn get_register_platform_signature(
        &self,
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

        // Ask the client to sign the given typed data
        Ok(self
            .signer
            .sign_typed_data(&create_platform, &self.get_domain())
            .await
            .unwrap())
    }
}
