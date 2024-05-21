use alloc::{string::String, vec, vec::Vec};
use core::marker::PhantomData;

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes},
    alloy_sol_types::sol,
    crypto::keccak,
    msg::{self},
    prelude::*,
    storage::{StorageAddress, StorageB256, StorageB32, StorageMap, StorageString},
};

pub trait PlatformParams {}

// Define events and errors in the contract
sol! {
    error NotPlatformOwner();
    error PlatformAlreadyExist();
    error PlatformDontExist();
    error InvalidMetadata();
}

#[derive(SolidityError)]
pub enum PlatformError {
    NotPlatformOwner(NotPlatformOwner),
    PlatformAlreadyExist(PlatformAlreadyExist),
    PlatformDontExist(PlatformDontExist),
    InvalidMetadata(InvalidMetadata),
}

/// Define the platform metadata
#[solidity_storage]
pub struct PlatformMetadata {
    name: StorageString,
    owner: StorageAddress,
    content_type: StorageB32,
    origin: StorageB256,
}

/// Define the type of the platform metadata, in an ideal world, it should be a sol type
/// TODO: When alloy-rs is updated on the stylus SDK, use a sol! macro to define the type
type PlatformMetadataType = (String, Address, FixedBytes<4>, FixedBytes<32>);

// Define the global conntract storage
#[solidity_storage]
pub struct PlatformContract<T: PlatformParams> {
    // The platform metadata storage (platform_id => PlatformMetadata)
    platform_data: StorageMap<FixedBytes<32>, PlatformMetadata>,
    phantom: PhantomData<T>,
}

/// Internal method stuff
impl<T: PlatformParams> PlatformContract<T> {
    /* -------------------------------------------------------------------------- */
    /*                                   Helpers                                  */
    /* -------------------------------------------------------------------------- */

    /// Check if a platform exist
    fn _platform_exist(&self, platform_id: FixedBytes<32>) -> bool {
        self.platform_data.get(platform_id).owner.is_empty()
    }

    /// Only allow the call for an existing platform
    pub fn only_existing_platform(&self, platform_id: FixedBytes<32>) -> Result<(), PlatformError> {
        if !self._platform_exist(platform_id) {
            return Err(PlatformError::PlatformDontExist(PlatformDontExist {}));
        }
        Ok(())
    }

    /// Get the platform owner
    pub fn get_platform_owner(
        &self,
        platform_id: FixedBytes<32>,
    ) -> Result<Address, PlatformError> {
        self.only_existing_platform(platform_id)?;
        Ok(self.platform_data.get(platform_id).owner.get())
    }

    /* -------------------------------------------------------------------------- */
    /*                         Platform  creation / update                        */
    /* -------------------------------------------------------------------------- */

    /// Create a new platform
    /// returns the created platform_id
    pub fn _create_platform(
        &mut self,
        name: String,
        owner: Address,
        content_type: FixedBytes<4>,
        origin: FixedBytes<32>,
    ) -> Result<FixedBytes<32>, PlatformError> {
        let platform_id = keccak(origin);
        // Ensure the platform doesn't already exist
        if self._platform_exist(platform_id) {
            return Err(PlatformError::PlatformAlreadyExist(PlatformAlreadyExist {}));
        }

        // Ensure name and owner aren't empty
        if name.is_empty() | owner.is_zero() {
            return Err(PlatformError::InvalidMetadata(InvalidMetadata {}));
        }

        // Get the right metadata setter
        let mut metadata = self.platform_data.setter(platform_id);
        // Create the new platform metadata
        metadata.name.set_str(name);
        metadata.owner.set(owner);
        metadata.content_type.set(content_type);
        metadata.origin.set(origin);

        // Return the created platform_id
        Ok(platform_id)
    }

    /// Update a platform metadata (could be name or owner)
    fn _update_platform_metadata(
        &mut self,
        platform_id: FixedBytes<32>,
        name: String,
        owner: Address,
    ) -> Result<(), PlatformError> {
        self.only_existing_platform(platform_id)?;

        // Ensure name isn't empty
        if name.is_empty() | owner.is_zero() {
            return Err(PlatformError::InvalidMetadata(InvalidMetadata {}));
        }
        // Get the right metadata setter
        let mut metadata = self.platform_data.setter(platform_id);
        // Create the new platform metadata
        metadata.name.set_str(name);
        metadata.owner.set(owner);
        // Return success
        Ok(())
    }
}

/// External method stuff
#[external]
impl<T: PlatformParams> PlatformContract<T> {
    /* -------------------------------------------------------------------------- */
    /*                          Platform update methods                           */
    /* -------------------------------------------------------------------------- */

    /// Update the platform metadata
    #[selector(name = "updatePlatformMetadata")]
    pub fn update_metadata(
        &mut self,
        platform_id: FixedBytes<32>,
        name: String,
        owner: Address,
    ) -> Result<(), PlatformError> {
        let caller = msg::sender();
        // Ensure the caller is the owner
        if !self.platform_data.get(platform_id).owner.get().eq(&caller) {
            return Err(PlatformError::NotPlatformOwner(NotPlatformOwner {}));
        }

        // Update the platform metadata
        self._update_platform_metadata(platform_id, name, owner)
    }

    /* -------------------------------------------------------------------------- */
    /*                           Platform read methods                            */
    /* -------------------------------------------------------------------------- */

    /// Get a platform metadata
    #[selector(name = "getPlatformMetadata")]
    #[view]
    pub fn get_platform_metadata(
        &self,
        platform_id: FixedBytes<32>,
    ) -> Result<PlatformMetadataType, Vec<u8>> {
        // Get the ptr to the platform metadata
        let ptr = self.platform_data.get(platform_id);
        // Return every field we are interested in
        Ok((
            // Classical metadata
            ptr.name.get_string(),
            ptr.owner.get(),
            ptr.content_type.get(),
            ptr.origin.get(),
        ))
    }
}
