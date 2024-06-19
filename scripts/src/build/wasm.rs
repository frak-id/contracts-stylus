use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{build::constants::WASM_TARGET_TRIPLE, errors::ScriptError, utils::command_success_or};

/// Global definition of the WasmBuilder trait
/// TODO: We should include config with optimisation steps and stuff here
pub struct WasmBuilder;
impl WasmBuilder {
    /// Full build of the specified `contract_profile`
    pub fn build_wasm(&self, contract_profile: &str) -> Result<PathBuf, ScriptError> {
        let current_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let workspace_path = current_dir
            .parent()
            .ok_or(ScriptError::ContractCompilation(String::from(
                "Could not find contracts-stylus directory",
            )))?;

        // Build the initial wasm file
        let wasm_file_path = self.build_initial_wasm(workspace_path, contract_profile)?;

        // Build the optimized wasm file
        let opt_wasm_file_path = self.build_opt_wasm(&wasm_file_path)?;

        // Output the resulting wasm file
        Ok(opt_wasm_file_path)
    }

    /// Build the initial wasm file
    fn build_initial_wasm(
        &self,
        workspace_path: &Path,
        contract_profile: &str,
    ) -> Result<PathBuf, ScriptError> {
        let mut build_cmd = Command::new("cargo");
        build_cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        // Set the working directory to the workspace root
        build_cmd.arg("-C");
        build_cmd.arg(workspace_path);
        // Invoke the build command
        build_cmd.arg("build");
        // Use the release profile
        build_cmd.arg("-r");
        // Build the contracts-stylus-stylus package
        build_cmd.arg("-p");
        build_cmd.arg(contract_profile);
        // Set the build target to WASM
        build_cmd.arg("--target");
        build_cmd.arg(WASM_TARGET_TRIPLE);
        // Set the Z flags, used to optimize the resulting binary size.
        build_cmd.args("-Z unstable-options -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort".split_whitespace());

        // Set aggressive optimisation flags
        env::set_var("RUSTFLAGS", "-C opt-level=3");
        command_success_or(build_cmd, "Failed to build contract WASM")?;
        env::remove_var("RUSTFLAGS");

        // Then, fetch the build directory
        let target_dir = workspace_path
            .join("target")
            .join(WASM_TARGET_TRIPLE)
            .join("release");

        // And find the target file inside it
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

        Ok(wasm_file_path)
    }

    /// Build the optimized wasm file
    fn build_opt_wasm(&self, wasm_file_path: &Path) -> Result<PathBuf, ScriptError> {
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
}
