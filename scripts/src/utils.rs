//! Utilities for the deploy scripts.

use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
    sync::Arc,
};

use crate::constants::WASM_TARGET_TRIPLE;
use ethers::{
    abi::Address,
    middleware::SignerMiddleware,
    providers::{Http, Middleware, Provider},
    signers::{LocalWallet, Signer},
    utils::get_contract_address,
};

use crate::errors::ScriptError;

/// An Ethers provider that uses a `LocalWallet` to generate signatures
/// & interfaces with the RPC endpoint over HTTP
pub type LocalWalletHttpClient = SignerMiddleware<Provider<Http>, LocalWallet>;

/// Sets up the address and client with which to instantiate a contract for testing,
/// reading in the private key, RPC url, and contract address from the environment.
pub async fn setup_client(
    priv_key: &str,
    rpc_url: &str,
) -> Result<Arc<LocalWalletHttpClient>, ScriptError> {
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;

    // Create the wallet from priv key
    let wallet = LocalWallet::from_str(priv_key)
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?;
    // Fetch chain id
    let chain_id = provider
        .get_chainid()
        .await
        .map_err(|e| ScriptError::ClientInitialization(e.to_string()))?
        .as_u64();
    // Bound the wallet to the provider, via a middleware
    let client = Arc::new(SignerMiddleware::new(
        provider,
        wallet.clone().with_chain_id(chain_id),
    ));

    Ok(client)
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
    client: Arc<LocalWalletHttpClient>,
) -> Result<Address, ScriptError> {
    // Get expected deployment address
    let deployer_address = client
        .default_sender()
        .ok_or(ScriptError::ClientInitialization(
            "client does not have sender attached".to_string(),
        ))?;
    let deployer_nonce = client
        .get_transaction_count(deployer_address, None /* block */)
        .await
        .map_err(|e| ScriptError::NonceFetching(e.to_string()))?;
    let deployed_address = get_contract_address(deployer_address, deployer_nonce);

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

    Ok(deployed_address)
}
