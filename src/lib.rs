// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

/// Import items from the SDK. The prelude contains common traits and macros.
use alloc::{string::String, vec, vec::Vec};
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, B128, U256},
    prelude::*,
    storage::{
        StorageAddress, StorageArray, StorageB256, StorageB32, StorageMap, StorageString,
        StorageU256,
    },
};

// Define the user data on the given plateform
#[solidity_storage]
pub struct UserPlatformData {
    ccu: StorageU256,
    u_block: StorageU256,
}

// Define the palteform metadata
#[solidity_storage]
pub struct PlatformMetadata {
    name: StorageString,
    owner: StorageAddress,
    content_type: StorageB32,
    origin: StorageB256,
    public_key: StorageArray<StorageU256, 4>,
}

// Define the global conntract storage
#[solidity_storage]
#[entrypoint]
pub struct ContentConsumptionContract {
    // The user activity storage (user => plateform_id => UserPlatformData)
    user_activity: StorageMap<Address, StorageMap<B128, UserPlatformData>>,
    // The plateform metadata storage (plateform_id => PlatformMetadata)
    platform_data: StorageMap<B128, PlatformMetadata>
}

/// Declare that `ContentConsumptionContract` is a contract with the following external methods.
#[external]
impl ContentConsumptionContract {

    /* -------------------------------------------------------------------------- */
    /*                           Plateform read methods                           */
    /* -------------------------------------------------------------------------- */

    /// Get a plateform metadatas
    /// TODO: Can the result be replaced by a struct or smth like that?
    #[selector(name = "getPlateformMetadatas")]
    pub fn get_plateform_metadatas(
        &self,
        plateform_id: B128,
    ) -> Result<
        (
            String,
            Address,
            FixedBytes<4>,
            FixedBytes<32>,
            (U256, U256, U256, U256),
        ),
        Vec<u8>,
    > {
        // Get the ptr to the plateform metadata
        let ptr = self.platform_data.get(plateform_id);
        // Return every field we are interested in
        Ok((
            // Classical metadatas
            ptr.name.get_string(),
            ptr.owner.get(),
            ptr.content_type.get(),
            ptr.origin.get(),
            // Public key
            (
                ptr.public_key.get(0).unwrap(),
                ptr.public_key.get(1).unwrap(),
                ptr.public_key.get(2).unwrap(),
                ptr.public_key.get(3).unwrap(),
            ),
        ))
    }
}
