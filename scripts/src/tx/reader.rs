use alloy::primitives::{Address, B256, U256};

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
