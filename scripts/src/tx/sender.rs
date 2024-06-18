use alloy::{
    network::TransactionBuilder,
    primitives::{Address, FixedBytes, TxHash, B256, U256},
    providers::Provider,
    rpc::types::TransactionRequest,
};
use tracing::info;

use crate::{
    errors::ScriptError,
    tx::{abi::IContentConsumptionContract, client::RpcProvider, typed_data::TypedDataSigner},
};

/// Init consumption contracts
pub async fn send_init_consumption_contract(
    contract: Address,
    owner: Address,
    client: RpcProvider,
) -> Result<TxHash, ScriptError> {
    // Build the tx
    let tx_request = TransactionRequest::default()
        .to(contract)
        .with_call(&IContentConsumptionContract::initializeCall { owner })
        .with_value(U256::from(0));

    // Send it
    let pending_tx = client
        .send_transaction(tx_request)
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!("Pending init transaction... {}", pending_tx.tx_hash());

    // Wait for the transaction to be included.
    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!("Init tx done on block: {}", receipt.block_number.unwrap());

    Ok(receipt.transaction_hash)
}

/// Create a new platform
pub async fn send_create_platform(
    contract_address: Address,
    name: String,
    origin: String,
    owner: Address,
    content_type: B256,
    client: RpcProvider,
) -> Result<(FixedBytes<32>, TxHash), ScriptError> {
    // Build our contract
    let contract = IContentConsumptionContract::new(contract_address, client.clone());

    let typed_data_signer = TypedDataSigner::new(client.clone(), contract_address).await?;

    // Build the signature
    let deadline = U256::MAX;
    let signature = typed_data_signer
        .get_register_platform_signature(owner, name.clone(), origin.clone(), deadline)
        .await?;

    // Craft the register platform tx and send it
    let register_platform_tx_builder = contract.registerPlatform(
        name,
        origin,
        owner,
        content_type,
        deadline,
        signature.v().y_parity_byte(),
        signature.r().into(),
        signature.s().into(),
    );

    // Predicate the platform id
    let register_platform_return = register_platform_tx_builder
        .call()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    let platform_id = register_platform_return._0;

    // Send it
    let register_platform_tx = register_platform_tx_builder
        .send()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;

    // Send it
    info!(
        "Pending create platform transaction... {}, estimated platform id: {:?}",
        &register_platform_tx.tx_hash(),
        platform_id
    );

    // Wait for the transaction to be included.
    let receipt = register_platform_tx
        .get_receipt()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!(
        "Create platform tx done on block: {}",
        receipt.block_number.unwrap()
    );

    // Read the smart contract
    let platform_name = contract
        .getPlatformName(platform_id)
        .call()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!("OnChain platform name: {:?}", platform_name._0);

    // Read the smart contract
    let platform_metadata = contract
        .getPlatformMetadata(platform_id)
        .call()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!(
        "OnChain platform metadata: owner: {:?}, content type: {:?}",
        platform_metadata._0, platform_metadata._1
    );

    Ok((platform_id, receipt.transaction_hash))
}

/// Send CCU on chain
pub async fn push_ccu(
    contract_address: Address,
    user: Address,
    platform_id: B256,
    consumption: U256,
    deadline: U256,
    v: u8,
    r: U256,
    s: U256,
    client: RpcProvider,
) -> Result<TxHash, ScriptError> {
    // Build our contract
    let contract = IContentConsumptionContract::new(contract_address, client);

    // Craft the register platform tx and send it
    let send_ccu_tx_builder = contract.pushCcu(
        user,
        platform_id,
        consumption,
        deadline,
        v,
        r.into(),
        s.into(),
    );

    // Test a call first
    let send_ccu_call = send_ccu_tx_builder
        .call_raw()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!("Push CCU call: {:?}", send_ccu_call);

    // Then send it
    let send_ccu_tx = send_ccu_tx_builder
        .send()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;

    // Send it
    info!("Push CCU transaction... {}", &send_ccu_tx.tx_hash());

    // Wait for the transaction to be included.
    let receipt = send_ccu_tx
        .get_receipt()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!(
        "Push CCU tx done on block: {}",
        receipt.block_number.unwrap()
    );

    Ok(receipt.transaction_hash)
}

/// Set a new validator
pub async fn set_validator(
    contract_address: Address,
    validator: Address,
    allowed: bool,
    client: RpcProvider,
) -> Result<TxHash, ScriptError> {
    // Build our contract
    let contract = IContentConsumptionContract::new(contract_address, client);

    // Craft the register platform tx and send it
    let allowed_validator_builder = contract.setAllowedValidator(validator, allowed);

    // Test a call first
    let set_validator_call = allowed_validator_builder
        .call_raw()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!("Set allowed validator call: {:?}", set_validator_call);

    // Then send it
    let set_allowed_validator_tx = allowed_validator_builder
        .send()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;

    // Send it
    info!(
        "Push validator on transaction... {}",
        &set_allowed_validator_tx.tx_hash()
    );

    // Wait for the transaction to be included.
    let receipt = set_allowed_validator_tx
        .get_receipt()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!(
        "Set validator tx done on block: {}",
        receipt.block_number.unwrap()
    );

    Ok(receipt.transaction_hash)
}
