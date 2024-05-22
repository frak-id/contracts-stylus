use std::{
    path::PathBuf,
    process::{Command, Stdio},
};

use alloy::{
    primitives::{keccak256, Address},
    providers::{Provider, WalletProvider},
};
use ethers::{prelude::U256, utils::rlp};

use crate::{errors::ScriptError, tx::client::RpcProvider, utils::command_success_or};

/// Deploy a built stylus contract
pub async fn deploy_contract(
    wasm_file_path: PathBuf,
    rpc_url: &str,
    priv_key: &str,
    client: RpcProvider,
) -> Result<Address, ScriptError> {
    // Predict the contract address
    let contract_address = predict_contract_address(client.clone()).await?;

    // TODO: A manual deployment should be prefered here (compress, then build calldata for deploy + activate and send that in a batch)
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

    Ok(contract_address)
}

/// Predict the contract address of the deployed contract
/// TODO: Should use alloy RLP here, but didn't manage to get it working nicely
/// This is not working:
/// ```
///    let mut out: Vec<u8> = Vec::new();
//     let list: [&dyn Encodable; 2] = [&signer.as_slice().to_vec(), &nonce];
//     alloy_rlp::encode_list::<_, dyn Encodable>(&list, &mut out);
/// ```
async fn predict_contract_address(client: RpcProvider) -> Result<Address, ScriptError> {
    // Get signer
    let signer = client.signer().default_signer().address();

    // Get the signer nonce
    let signer_nonce = client
        .get_transaction_count(signer.clone())
        .await
        .map_err(|e| ScriptError::NonceFetching(e.to_string()))?;

    // Ethers RLP
    let mut stream = rlp::RlpStream::new();
    stream.begin_list(2);
    stream.append(&signer.to_vec());
    stream.append(&U256::from(signer_nonce));
    let hash = keccak256(&stream.out());

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&hash[12..]);
    Ok(Address::from(bytes))
}
