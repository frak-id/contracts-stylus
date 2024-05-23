use alloy::{
    primitives::{keccak256, Address, StorageValue, B256, U256},
    providers::Provider,
};

use crate::{
    errors::ScriptError,
    tx::{abi::IContentConsumptionContract, client::RpcProvider},
};

/// Get the current contract domain separator
pub async fn get_domain_separator(
    contract_address: Address,
    client: RpcProvider,
) -> Result<B256, ScriptError> {
    // Build our contract
    let contract = IContentConsumptionContract::new(contract_address, client);

    // Read the smart contract
    let domain_separator = contract
        .domainSeparator()
        .call()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;

    Ok(domain_separator._0)
}

/// Get the current contract domain separator
pub async fn get_nonce_on_platform(
    contract_address: Address,
    client: RpcProvider,
    user: Address,
    platform_id: B256,
) -> Result<U256, ScriptError> {
    // Build our contract
    let contract = IContentConsumptionContract::new(contract_address, client);

    // Read the smart contract
    let nonce = contract
        .getNonceForPlatform(user, platform_id)
        .call()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;

    Ok(nonce._0)
}

/// Get the current contract domain separator
pub async fn read_ccu_from_storage(
    contract_address: Address,
    client: RpcProvider,
    user: Address,
    platform_id: B256,
) -> Result<U256, ScriptError> {
    // user => platformId => UserConsumption (first 32 bytes is ccu)

    // Perform the first keccak
    let mut bytes_to_hash = [0u8; 64];
    bytes_to_hash[0..32].copy_from_slice(&user.into_word().to_vec());
    bytes_to_hash[32..64].copy_from_slice(&B256::ZERO.to_vec());
    let user_ptr = keccak256(bytes_to_hash);
    // Perform the second keccak
    bytes_to_hash[0..32].copy_from_slice(&platform_id.to_vec());
    bytes_to_hash[32..64].copy_from_slice(&user_ptr.to_vec());
    let storage_ptr = keccak256(bytes_to_hash);

    // Read the smart contract
    let storage: StorageValue = client
        .get_storage_at(contract_address, storage_ptr.into())
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;

    Ok(storage)
}
