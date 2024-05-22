use alloy::{
    network::TransactionBuilder,
    primitives::{aliases::B32, keccak256, Address, TxHash, U256},
    providers::Provider,
    rpc::types::eth::TransactionRequest,
};
use tracing::info;

use crate::{
    errors::ScriptError,
    tx::{abi::IContentConsumptionContract, client::RpcProvider},
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
    content_type: B32,
    client: RpcProvider,
) -> Result<TxHash, ScriptError> {
    // Perform a hash on the origin
    let origin_hash = keccak256(origin);

    // Build our contract
    let contract = IContentConsumptionContract::new(contract_address, client);

    // Craft the register platform tx and send it
    let register_platform_tx_builder =
        contract.registerPlatform(name, owner, content_type, origin_hash);

    // Send it
    let register_platform_tx = register_platform_tx_builder
        .send()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;

    // Send it
    info!(
        "Pending create platform transaction... {}",
        &register_platform_tx.tx_hash()
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
        .getPlatformName(keccak256(origin_hash))
        .call()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!("OnChain platform name: {:?}", platform_name._0);

    // Read the smart contract
    let platform_metadata = contract
        .getPlatformMetadata(keccak256(origin_hash))
        .call()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!(
        "OnChain platform metadata: owner: {:?}, content type: {:?}, origin {:?}",
        platform_metadata._0, platform_metadata._1, platform_metadata._2
    );

    Ok(receipt.transaction_hash)
}

/*
TODO:
 - Craft a command to extract domain etc
 - Create a small typescript stuff that will try to push CCU
 */
