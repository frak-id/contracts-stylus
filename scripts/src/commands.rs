use alloy::{
    primitives::{aliases::B32, keccak256, private::getrandom::getrandom, Address, B256},
    providers::WalletProvider,
};
use tracing::info;

use crate::{
    cli::{CreatePlatformArgs, DeployContractsArgs},
    constants::OUTPUT_FILE,
    deploy::deploy::deploy_contract,
    errors::ScriptError,
    output_writer::{read_output_file, write_output_file, OutputKeys},
    tx::{
        client::RpcProvider,
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
        client,
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
