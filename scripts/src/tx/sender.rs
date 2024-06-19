use alloy::{
    network::TransactionBuilder,
    primitives::{Address, TxHash, B256, U256},
    providers::Provider,
    rpc::types::TransactionRequest,
};
use tracing::info;

use crate::{
    errors::ScriptError,
    tx::{abi::IContentConsumptionContract, client::RpcProvider},
};

/// Init consumption contracts-stylus
pub async fn send_init_consumption_contract(
    contract: Address,
    owner: Address,
    content_registry: Address,
    frak_id: U256,
    client: RpcProvider,
) -> Result<TxHash, ScriptError> {
    // Build the tx
    let tx_request = TransactionRequest::default()
        .to(contract)
        .with_call(&IContentConsumptionContract::initializeCall {
            owner,
            contentRegistryAddress: content_registry,
            frakContentId: frak_id,
        })
        .with_value(U256::from(0));
    info!(
        "Sending init call with owner: {:?}, content registry {:?}, frak_id {:?}",
        owner, content_registry, frak_id
    );

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

/// Send CCU on chain
pub async fn push_ccu(
    contract_address: Address,
    channel_id: B256,
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
    let send_ccu_tx_builder =
        contract.pushCcu(channel_id, consumption, deadline, v, r.into(), s.into());

    // Test a call first
    let send_ccu_call = send_ccu_tx_builder
        .call_raw()
        .await
        .map_err(|e| ScriptError::ContractSimulation(e.to_string()))?;
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
// 0xceea21b6
// 0x609c6d7f
// b0619b27c165cb8eb016dbfdcdbebed113641649d139147c3130c58eec9ef748
// 000000000000000000000000f2502c2f6bc19cdbdd1b8565ad6fbc29efe4f464

// 0xb0619b27c165cb8eb016dbfdcdbebed113641649d139147c3130c58eec9ef748
// 0x0000000000000000000000000000000000000000000000000000000000000000
