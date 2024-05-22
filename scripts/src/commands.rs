use std::env;

use alloy::{
    core::sol,
    primitives::{aliases::B32, keccak256, private::getrandom::getrandom, Address, B256, U256},
    providers::{Provider, WalletProvider},
    signers::{k256::ecdsa::SigningKey, wallet::Wallet, Signer},
    sol_types::eip712_domain,
};
use tracing::info;

use crate::{
    cli::{CreatePlatformArgs, DeployContractsArgs, SendTestCcuArgs},
    constants::OUTPUT_FILE,
    deploy::deploy::deploy_contract,
    errors::ScriptError,
    output_writer::{read_output_file, write_output_file, OutputKeys},
    tx::{
        client::RpcProvider,
        reader::get_nonce_on_platform,
        sender::{send_create_platform, send_init_consumption_contract},
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
    let deployer_address = client.signer().default_signer().address();
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

    // Update args.origin to also include a random number
    let mut buf = [0u8; 32];
    getrandom(&mut buf).unwrap();
    let origin = format!("{}-{}", args.origin, B256::from(buf));

    // Compute the origin hash
    let origin_hash = keccak256(&origin);

    // Parse the owner address
    let owner_address = args.owner.parse::<Address>().unwrap();

    // Send the tx
    let tx_hash = send_create_platform(
        deployed_address,
        "test".to_string(),
        origin,
        owner_address,
        B32::from(args.content_type),
        client.clone(),
    )
    .await?;

    let tx_key = format!("create_platform_{origin_hash:x}");

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

    info!(
        "Crafting test CCU for user: {} and platform: {}",
        user_address, platform_id
    );

    // Get the current user nonce on the given platform
    let nonce =
        get_nonce_on_platform(deployed_address, client.clone(), user_address, platform_id).await?;

    info!("User nonce: {}", nonce);

    sol! {
        struct ValidateConsumption {
            address user;
            bytes32 platformId;
            uint256 addedConsumption;
            uint256 nonce;
            uint256 deadline;
        }
    }

    // Fetch the chain id
    let chain_id = client.get_chain_id().await.unwrap();

    let domain = eip712_domain! {
        name: "ContentConsumption",
        version: "0.0.1",
        chain_id: chain_id,
        verifying_contract: deployed_address,
    };
    let validate_consumption = ValidateConsumption {
        user: user_address,
        platformId: platform_id,
        addedConsumption: U256::from(10u64),
        nonce,
        deadline: U256::MAX,
    };

    let signer = env::var("PRIVATE_KEY")
        .unwrap()
        .parse::<Wallet<SigningKey>>()
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    // Ask the client to sign the given typed data
    let signed_result = signer
        .sign_typed_data(&validate_consumption, &domain)
        .await
        .unwrap();
    info!("Signed result: {:?}", signed_result.as_bytes());

    Ok(())
}
