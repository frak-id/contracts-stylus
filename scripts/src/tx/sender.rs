use alloy::{
    network::TransactionBuilder,
    primitives::{aliases::B32, keccak256, Address, TxHash, U256},
    providers::Provider,
    rpc::types::eth::TransactionRequest,
};
use tracing::info;

use crate::{
    errors::ScriptError,
    tx::{
        abi::{initializeCall, registerPlatformCall},
        client::RpcProvider,
    },
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
        .with_call(&initializeCall { owner })
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

/// Create a new plateform
pub async fn send_create_platform(
    contract: Address,
    name: String,
    origin: String,
    owner: Address,
    content_type: B32,
    client: RpcProvider,
) -> Result<TxHash, ScriptError> {
    // Perform a hash on the origin
    let origin_hash = keccak256(origin);

    // Build the tx
    let tx_request = TransactionRequest::default()
        .to(contract)
        .with_call(&registerPlatformCall {
            name,
            owner,
            content_type,
            origin: origin_hash,
        })
        .with_value(U256::from(0));

    // Send it
    let pending_tx = client
        .send_transaction(tx_request)
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!(
        "Pending create platform transaction... {}",
        pending_tx.tx_hash()
    );

    // Wait for the transaction to be included.
    let receipt = pending_tx
        .get_receipt()
        .await
        .map_err(|e| ScriptError::ContractInteraction(e.to_string()))?;
    info!(
        "Create platform tx done on block: {}",
        receipt.block_number.unwrap()
    );

    Ok(receipt.transaction_hash)
}
