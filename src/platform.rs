use alloc::{string::String, vec, vec::Vec};
/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, B128, U256},
    prelude::*,
    storage::{
        StorageAddress, StorageArray, StorageB256, StorageB32, StorageMap, StorageString,
        StorageU256,
    },
};

/// Define the palteform metadata
#[solidity_storage]
pub struct PlatformMetadata {
    name: StorageString,
    owner: StorageAddress,
    content_type: StorageB32,
    origin: StorageB256,
    public_key: StorageArray<StorageU256, 4>,
}

/// Define the type of the plateform metadata, in an ideal world, it should be a sol type
/// TODO: When alloy-rs is updated on the stylus SDK, use a sol! macro to define the type
type PlatformMetadataType = (
    String,
    Address,
    FixedBytes<4>,
    FixedBytes<32>,
    (U256, U256, U256, U256),
);

// Define the global conntract storage
#[solidity_storage]
pub struct PlateformContract {
    // The plateform metadata storage (plateform_id => PlatformMetadata)
    platform_data: StorageMap<B128, PlatformMetadata>,
}

/// Declare that `ContentConsumptionContract` is a contract with the following external methods.
#[external]
impl PlateformContract {
    /* -------------------------------------------------------------------------- */
    /*                           Plateform read methods                           */
    /* -------------------------------------------------------------------------- */

    /// Get a plateform metadatas
    /// TODO: Pub key could be a B1024, should find a way to concat it to that type (but not aliased from the stylus SDK)
    #[selector(name = "getPlateformMetadatas")]
    pub fn get_plateform_metadatas(
        &self,
        plateform_id: B128,
    ) -> Result<PlatformMetadataType, Vec<u8>> {
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
