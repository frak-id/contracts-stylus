// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

// Utility functions and helpers used across the library
mod consumption;
mod platform;

pub use consumption::ConsumptionContract;
pub use platform::PlateformContract;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::vec;
use stylus_sdk::prelude::*;

// Define the global conntract storage
#[solidity_storage]
#[entrypoint]
pub struct ContentConsumptionContract {
    #[borrow]
    plateform: PlateformContract,
    #[borrow]
    consumption: ConsumptionContract,
}

/// Declare that `ContentConsumptionContract` is a contract with the following external methods.
#[external]
#[inherit(PlateformContract, ConsumptionContract)]
impl ContentConsumptionContract {

    /* -------------------------------------------------------------------------- */
    /*                          Consumption read methods                          */
    /* -------------------------------------------------------------------------- */

    /// Get the user consumption on a content
    #[selector(name = "test")]
    pub fn test(
        &self
    ) -> Result<String, Vec<u8>> {
        // Return every field we are interested in
        Ok("test".to_string())
    }

}
