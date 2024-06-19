use alloy::{
    primitives::{Address, B256}
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
