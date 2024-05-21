//! Utilities for the deploy scripts.

use std::{
    env, fs,
    fs::File,
    io::Read,
    path::PathBuf,
    process::{Command, Stdio},
};

use alloy_network::{Ethereum, EthereumSigner};
use alloy_primitives::Address;
use alloy_provider::{
    fillers::{ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, SignerFiller},
    Identity, Provider, ProviderBuilder, ReqwestProvider, WalletProvider,
};
use alloy_signer::k256::ecdsa::SigningKey;
use alloy_signer_wallet::Wallet;
use json::JsonValue;
use reqwest::{Client, Url};
use tracing::info;

use crate::{constants::WASM_TARGET_TRIPLE, errors::ScriptError};

/// Re-export from alloy recommend filter
type RecommendFiller =
    JoinFill<JoinFill<JoinFill<Identity, GasFiller>, NonceFiller>, ChainIdFiller>;

/// An Ethers provider that uses a `LocalWallet` to generate signatures
/// & interfaces with the RPC endpoint over HTTP
pub type RpcProvider = FillProvider<
    JoinFill<RecommendFiller, SignerFiller<EthereumSigner>>,
    ReqwestProvider,
    alloy_transport_http::Http<Client>,
    Ethereum,
>;

/// Sets up the address and client with which to instantiate a contract for testing,
/// reading in the private key, RPC url, and contract address from the environment.
pub async fn setup_client(priv_key: &str, rpc_url: &str) -> Result<RpcProvider, ScriptError> {
    // Create our signer
    let signer = priv_key
        .parse::<Wallet<SigningKey>>()
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    // Create our provider with the rpc client + signer
    let provider: FillProvider<
        JoinFill<RecommendFiller, SignerFiller<EthereumSigner>>,
        ReqwestProvider,
        alloy_transport_http::Http<Client>,
        Ethereum,
    > = ProviderBuilder::new()
        .with_recommended_fillers()
        .signer(EthereumSigner::from(signer))
        .on_http(rpc_url.parse::<Url>().unwrap());

    // Fetch chain id
    let chain_id = provider
        .get_chain_id()
        .await
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    info!("Build client on chain ID: {}", chain_id);

    Ok(provider)
}

/// Executes a command, returning an error if the command fails
fn command_success_or(mut cmd: Command, err_msg: &str) -> Result<(), ScriptError> {
    println!("Running command: {:?}", cmd);
    if !cmd
        .output()
        .map_err(|e| ScriptError::ContractCompilation(e.to_string()))?
        .status
        .success()
    {
        Err(ScriptError::ContractCompilation(String::from(err_msg)))
    } else {
        Ok(())
    }
}

/// Builds the Stylus contract, returning the path to the compiled WASM file
pub fn build_stylus_contract() -> Result<PathBuf, ScriptError> {
    let current_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_path = current_dir
        .parent()
        .ok_or(ScriptError::ContractCompilation(String::from(
            "Could not find contracts directory",
        )))?;

    // Run initial WASM compilation
    let mut build_cmd = Command::new("cargo");
    build_cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    // Set the working directory to the workspace root
    build_cmd.arg("-C");
    build_cmd.arg(workspace_path);
    // Invoke the build command
    build_cmd.arg("build");
    // Use the release profile
    build_cmd.arg("-r");
    // Build the contracts-stylus package
    build_cmd.args("-p frak-contracts-stylus".split_whitespace());
    // Set the build target to WASM
    build_cmd.arg("--target");
    build_cmd.arg(WASM_TARGET_TRIPLE);
    // Set the Z flags, used to optimize the resulting binary size.
    build_cmd.args("-Z unstable-options -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort".split_whitespace());

    // Set aggressive optimisation flags
    env::set_var("RUSTFLAGS", "-C opt-level=3");

    command_success_or(build_cmd, "Failed to build contract WASM")?;

    env::remove_var("RUSTFLAGS");

    let target_dir = workspace_path
        .join("target")
        .join(WASM_TARGET_TRIPLE)
        .join("release");

    println!("Target dir: {:?}", target_dir);

    // Run wasm-opt, to optimise the resulting binary
    let wasm_file_path = fs::read_dir(target_dir)
        .map_err(|e| ScriptError::ContractCompilation(e.to_string()))?
        .find_map(|entry| {
            let path = entry.ok()?.path();
            path.extension()
                .is_some_and(|ext| ext == "wasm")
                .then_some(path)
        })
        .ok_or(ScriptError::ContractCompilation(String::from(
            "Could not find contract WASM file",
        )))?;

    let opt_wasm_file_path = wasm_file_path.with_extension("wasm.opt");

    let mut opt_cmd = Command::new("wasm-opt");
    opt_cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    opt_cmd.arg(wasm_file_path);
    opt_cmd.arg("-o");
    opt_cmd.arg(opt_wasm_file_path.clone());
    opt_cmd.arg("-O4"); // Aggressive optimization flag

    command_success_or(opt_cmd, "Failed to optimize contract WASM")?;

    Ok(opt_wasm_file_path)
}

/// Deploys the given compiled Stylus contract, saving its deployment address
pub async fn deploy_stylus_contract(
    wasm_file_path: PathBuf,
    rpc_url: &str,
    priv_key: &str,
    client: RpcProvider,
) -> Result<(), ScriptError> {
    // Extract first signer from the iterator
    let deployer_address = client.signer().default_signer().address();
    info!("Deployer address: {:#x}", deployer_address);

    let deployer_nonce = client
        .get_transaction_count(deployer_address.clone())
        .await
        .map_err(|e| ScriptError::NonceFetching(e.to_string()))?;
    info!("Current deployer nonce: {:#x}", deployer_nonce);
    // let deployed_address = get_contract_address([deployer_address, deployer_nonce]);
    // info!("Computed address: {:#x}", deployed_address);

    // Run deploy command
    let mut deploy_cmd = Command::new("cargo");
    deploy_cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    deploy_cmd.args("stylus deploy".split_whitespace());
    deploy_cmd.arg("--wasm-file");
    deploy_cmd.arg(&wasm_file_path);
    deploy_cmd.arg("-e");
    deploy_cmd.arg(rpc_url);
    deploy_cmd.arg("--private-key");
    deploy_cmd.arg(priv_key);

    command_success_or(deploy_cmd, "Failed to deploy Stylus contract")?;

    // Write the deployed address to the file
    // write_deployed_address("deployed.json", "consumption", deployed_address)?;

    Ok(())
}

/// Writes the given address for the deployed contract
pub fn write_deployed_address(
    file_path: &str,
    contract_key: &str,
    address: Address,
) -> Result<(), ScriptError> {
    // If the file doesn't exist, create it
    if !PathBuf::from(file_path).exists() {
        fs::write(file_path, "{}").map_err(|e| ScriptError::JsonOutputError(e.to_string()))?;
    }

    // Read the current file contents
    let mut file_contents = String::new();
    File::open(file_path)
        .map_err(|e| ScriptError::JsonOutputError(e.to_string()))?
        .read_to_string(&mut file_contents)
        .map_err(|e| ScriptError::JsonOutputError(e.to_string()))?;

    // Parse it's json content into objects
    let mut parsed_json =
        json::parse(&file_contents).map_err(|e| ScriptError::JsonOutputError(e.to_string()))?;

    // Update the contract address
    parsed_json[contract_key] = JsonValue::String(format!("{address:#x}"));

    // Write the updated json back to the file
    fs::write(file_path, json::stringify_pretty(parsed_json, 4))
        .map_err(|e| ScriptError::JsonOutputError(e.to_string()))?;

    Ok(())
}

/*pub fn get_contract_address(data: &[&dyn Encodable]) -> Address {
    // Array of stuff to encode
    let mut out = BytesMut::new();
    // Perform RLP encoding
    encode_list(data, &mut out);

    let hash = keccak256(&out);

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Address::from(bytes)
}*/
