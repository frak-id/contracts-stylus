use alloy::{
    primitives::{Address, B256, U256},
    providers::WalletProvider,
};
use tracing::info;

use crate::{
    cli::{DeployContractsArgs, SendTestCcuArgs},
    constants::OUTPUT_FILE,
    deploy::deploy::deploy_contract,
    errors::ScriptError,
    output_writer::{read_output_file, write_output_file, OutputKeys},
    tx::{
        client::RpcProvider,
        sender::{push_ccu, send_init_consumption_contract},
        typed_data::TypedDataSigner,
    },
};

/// Deploy the Frak consumption contracts-stylus
pub async fn deploy_contracts(
    args: DeployContractsArgs,
    client: RpcProvider,
) -> Result<(), ScriptError> {
    // Build the contracts-stylus
    info!("Building contracts-stylus...");
    let builder = crate::build::wasm::WasmBuilder {};
    let wasm_file_path = builder.build_wasm("frak-contracts-stylus")?;
    info!("Built with success");

    // Deploy them
    info!("Deploying contracts-stylus...");
    let contract_address = deploy_contract(wasm_file_path, client.clone()).await?;
    write_output_file(
        OUTPUT_FILE,
        OutputKeys::Deployment { key: "consumption" },
        contract_address,
    )?;
    info!("Deployed with success");

    // Init the contracts-stylus
    info!("Init contracts-stylus...");
    let content_registry_address = args.content_registry.parse::<Address>().unwrap();
    let frak_id = args.frak_id.parse::<U256>().unwrap();
    let deployer_address = client.wallet().default_signer().address();
    let tx_hash = send_init_consumption_contract(
        contract_address,
        deployer_address,
        content_registry_address,
        frak_id,
        client.clone(),
    )
    .await?;
    write_output_file(
        OUTPUT_FILE,
        OutputKeys::Init { key: "consumption" },
        tx_hash,
    )?;

    Ok(())
}

/// Send some test CCU
pub async fn send_test_ccu(args: SendTestCcuArgs, client: RpcProvider) -> Result<(), ScriptError> {
    // Fetch contract address from json
    let deployed_address =
        read_output_file(OUTPUT_FILE, OutputKeys::Deployment { key: "consumption" })?
            .parse::<Address>()
            .unwrap();

    // Parse provided args
    let channel_id = args.channel_id.parse::<B256>().unwrap();

    info!("Crafting test CCU for channel: {}", channel_id);

    // Build deadline + added consumption
    let deadline = U256::MAX;
    let added_consumption = U256::from(1);

    // Get our typed data signer
    let typed_data_signer = TypedDataSigner::new(client.clone(), deployed_address).await?;

    let user = client.wallet().default_signer().address();
    // Get the validate consumption signature
    let signed_result = typed_data_signer
        .get_validate_consumption_signature(user, channel_id, added_consumption, deadline)
        .await?;

    // Send the tx
    push_ccu(
        deployed_address,
        channel_id,
        added_consumption,
        deadline,
        signed_result.v().y_parity_byte(),
        signed_result.r(),
        signed_result.s(),
        client.clone(),
    )
    .await?;

    Ok(())
}
