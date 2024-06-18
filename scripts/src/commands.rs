use alloy::{
    primitives::{Address, B256, U256},
    providers::WalletProvider,
};
use tracing::info;

use crate::{
    cli::{CreatePlatformArgs, DeployContractsArgs, SendTestCcuArgs, SetValidatorArgs},
    constants::OUTPUT_FILE,
    deploy::deploy::deploy_contract,
    errors::ScriptError,
    output_writer::{read_output_file, write_output_file, OutputKeys},
    tx::{
        client::RpcProvider,
        reader::read_ccu_from_storage,
        sender::{push_ccu, send_create_platform, send_init_consumption_contract, set_validator},
        typed_data::TypedDataSigner,
    },
};

/// Deploy the Frak consumption contracts
pub async fn deploy_contracts(
    _args: DeployContractsArgs,
    client: RpcProvider,
) -> Result<(), ScriptError> {
    // Build the contracts
    info!("Building contracts...");
    let builder = crate::build::wasm::WasmBuilder {};
    let wasm_file_path = builder.build_wasm("frak-contracts-stylus")?;
    info!("Built with success");

    // Deploy them
    info!("Deploying contracts...");
    let contract_address = deploy_contract(wasm_file_path, client.clone()).await?;
    write_output_file(
        OUTPUT_FILE,
        OutputKeys::Deployment { key: "consumption" },
        contract_address,
    )?;
    info!("Deployed with success");

    // Init the contracts
    info!("Init contracts...");
    let deployer_address = client.wallet().default_signer().address();
    let tx_hash =
        send_init_consumption_contract(contract_address, deployer_address, client.clone()).await?;
    write_output_file(
        OUTPUT_FILE,
        OutputKeys::Init { key: "consumption" },
        tx_hash,
    )?;

    Ok(())
}
/// Init consumption contracts
pub async fn create_platform(
    args: CreatePlatformArgs,
    client: RpcProvider,
) -> Result<(), ScriptError> {
    // Fetch contract address from json
    let deployed_address =
        read_output_file(OUTPUT_FILE, OutputKeys::Deployment { key: "consumption" })?
            .parse::<Address>()
            .unwrap();

    // Extract platform origin
    let origin = args.origin;

    // Parse the owner address
    let owner_address = args.owner.parse::<Address>().unwrap();

    // Send the tx
    info!(
        "Sending create platform tx..., content type: {}",
        args.content_type
    );
    let (platform_id, tx_hash) = send_create_platform(
        deployed_address,
        origin.clone(),
        origin,
        owner_address,
        B256::from(args.content_type),
        client.clone(),
    )
    .await?;

    let tx_key = format!("create_platform_{platform_id:x}");

    // Write to the output file
    write_output_file(
        OUTPUT_FILE,
        OutputKeys::Tx {
            key: "consumption",
            tx_key,
        },
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
    let user_address = args.user.parse::<Address>().unwrap();
    let platform_id = args.platform_id.parse::<B256>().unwrap();

    // Read the ccu from direct storage
    let ccu_from_storage =
        read_ccu_from_storage(deployed_address, client.clone(), user_address, platform_id).await?;
    info!("CCU from storage: {}", ccu_from_storage);

    info!(
        "Crafting test CCU for user: {} and platform: {}",
        user_address, platform_id
    );

    // Build deadline + added consumption
    let deadline = U256::MAX;
    let added_consumption = U256::from(1);

    // Get our typed data signer
    let typed_data_signer = TypedDataSigner::new(client.clone(), deployed_address).await?;

    // Get the validate consumption signature
    let signed_result = typed_data_signer
        .get_validate_consumption_signature(user_address, platform_id, added_consumption, deadline)
        .await?;
    info!("Signed result: {:?}", signed_result.as_bytes());

    // Send the tx
    push_ccu(
        deployed_address,
        user_address,
        platform_id,
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

/// Push a new validator
pub async fn send_set_validator(
    args: SetValidatorArgs,
    client: RpcProvider,
) -> Result<(), ScriptError> {
    // Fetch contract address from json
    let deployed_address =
        read_output_file(OUTPUT_FILE, OutputKeys::Deployment { key: "consumption" })?
            .parse::<Address>()
            .unwrap();

    // Parse provided args
    let validator_address = args.validator.parse::<Address>().unwrap();

    // Send the tx
    set_validator(
        deployed_address,
        validator_address,
        args.allowed,
        client.clone(),
    )
    .await?;

    Ok(())
}
