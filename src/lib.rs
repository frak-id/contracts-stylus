// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

// Utility functions and helpers used across the library
mod consumption;
mod owned;
mod platform;

pub use consumption::{ConsumptionContract, UserConsumptionParams};
pub use owned::{Owned, OwnedParams};
pub use platform::{PlateformContract, PlateformParams};

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::vec;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    prelude::*,
};

struct CoreParam;

impl OwnedParams for CoreParam {}
impl PlateformParams for CoreParam {}
impl UserConsumptionParams for CoreParam {}

// Define the global conntract storage
#[solidity_storage]
#[entrypoint]
pub struct ContentConsumptionContract {
    #[borrow]
    plateform: PlateformContract<CoreParam>,
    #[borrow]
    consumption: ConsumptionContract<CoreParam>,
    #[borrow]
    owned: Owned<CoreParam>,
}

/// Declare that `ContentConsumptionContract` is a contract with the following external methods.
#[external]
#[inherit(PlateformContract<CoreParam>, ConsumptionContract<CoreParam>, Owned<CoreParam>)]
impl ContentConsumptionContract {
    /* -------------------------------------------------------------------------- */
    /*                                 Constructor                                */
    /* -------------------------------------------------------------------------- */

    /// Initialize the contract with an owner.
    /// TODO: No constructor poossible atm, so going with init method called during contract creation via multicall
    /// See: https://github.com/OffchainLabs/stylus-sdk-rs/issues/99
    pub fn initialize(&mut self, owner: Address) -> Result<(), Vec<u8>> {
        // Init the contract with the given owner
        self.owned.initialize(owner)?;

        // Return the success
        Ok(())
    }

    /* -------------------------------------------------------------------------- */
    /*                           Register new plateform                           */
    /* -------------------------------------------------------------------------- */

    /// Register a new plateform (only callable by the owner)
    pub fn register_plateform(
        &mut self,
        name: String,
        owner: Address,
        content_type: FixedBytes<4>,
        origin: FixedBytes<32>,
        public_key: [U256; 4],
    ) -> Result<(), Vec<u8>> {
        // Ensure that the caller is the owner
        self.owned.only_owner()?;

        // Create the plateform
        self.plateform
            ._create_plateform(name, owner, content_type, origin, public_key)?;

        // Return the success
        Ok(())
    }

    /*
     * TODO: -> Option to handle CCU
     *  -> Pushing new CCU should receive base CCU data (new amount), and check against a BLS signature if the owner of the plateform allowed it
     */
}
