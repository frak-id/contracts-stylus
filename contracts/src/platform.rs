use alloc::string::String;
use core::marker::PhantomData;

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes},
    alloy_sol_types::sol,
    crypto::keccak,
    evm,
    msg::{self},
    prelude::*,
    storage::{StorageAddress, StorageB32, StorageMap, StorageString},
};

use crate::errors::{
    Errors, InvalidMetadata, NotPlatformOwner, PlatformAlreadyExist, PlatformDontExist,
};

pub trait PlatformParams {}

// Define events and errors in the contract
sol! {
    event PlatformRegistered(bytes32 indexed platformId, address owner, bytes4 contentType);
}

/// Define the platform metadata
#[solidity_storage]
pub struct PlatformMetadata {
    name: StorageString,
    origin: StorageString,
    owner: StorageAddress,
    content_type: StorageB32,
}

// Define the global contract storage
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
    fn is_not_existent_platform(&self, platform_id: FixedBytes<32>) -> bool {
        self.platform_data.get(platform_id).owner.get().is_zero()
    }

    /// Only allow the call for an existing platform
    pub fn only_existing_platform(&self, platform_id: FixedBytes<32>) -> Result<(), Errors> {
        if self.is_not_existent_platform(platform_id) {
            return Err(Errors::PlatformDontExist(PlatformDontExist {}));
        }
        Ok(())
    }

    /// Get the platform owner
    pub fn get_platform_owner(&self, platform_id: FixedBytes<32>) -> Address {
        self.platform_data.get(platform_id).owner.get()
    }

    /* -------------------------------------------------------------------------- */
    /*                         Platform  creation / update                        */
    /* -------------------------------------------------------------------------- */

    /// Create a new platform
    /// returns the created platform_id
    pub fn create_platform(
        &mut self,
        name: String,
        origin: String,
        owner: Address,
        content_type: FixedBytes<4>,
    ) -> Result<FixedBytes<32>, Errors> {
        let platform_id = keccak(&origin);
        // Ensure the platform doesn't already exist
        if !self.is_not_existent_platform(platform_id) {
            return Err(Errors::PlatformAlreadyExist(PlatformAlreadyExist {}));
        }

        // Ensure name and owner aren't empty
        if name.is_empty() | owner.is_zero() {
            return Err(Errors::InvalidMetadata(InvalidMetadata {}));
        }

        // Get the right metadata setter
        let mut metadata = self.platform_data.setter(platform_id);
        // Create the new platform metadata
        metadata.name.set_str(name);
        metadata.origin.set_str(origin);
        metadata.owner.set(owner);
        metadata.content_type.set(content_type);

        // Emit the event
        evm::log(PlatformRegistered {
            platformId: platform_id.0,
            owner,
            contentType: content_type.0,
        });

        // Return the created platform_id
        Ok(platform_id)
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
    ) -> Result<(), Errors> {
        self.only_existing_platform(platform_id)?;

        let caller = msg::sender();
        // Ensure the caller is the owner
        if !self.platform_data.get(platform_id).owner.get().eq(&caller) {
            return Err(Errors::NotPlatformOwner(NotPlatformOwner {}));
        }

        // Ensure name isn't empty
        if name.is_empty() | owner.is_zero() {
            return Err(Errors::InvalidMetadata(InvalidMetadata {}));
        }

        // Get the right metadata setter
        let mut metadata = self.platform_data.setter(platform_id);
        // Create the new platform metadata
        metadata.name.set_str(name);
        metadata.owner.set(owner);
        // Return success
        Ok(())
    }

    /* -------------------------------------------------------------------------- */
    /*                           Platform read methods                            */
    /* -------------------------------------------------------------------------- */

    /// Get a platform metadata
    /// TODO: Adding a string to the output create some strange output data (including string offset inside the output wtf)
    #[selector(name = "getPlatformMetadata")]
    #[view]
    pub fn get_platform_metadata(
        &self,
        platform_id: FixedBytes<32>,
    ) -> Result<(Address, FixedBytes<4>), Errors> {
        // Get the ptr to the platform metadata
        let ptr = self.platform_data.get(platform_id);
        // Return every field we are interested in
        Ok((
            // Classical metadata
            ptr.owner.get(),
            ptr.content_type.get(),
        ))
    }

    /// Get a platform name
    #[selector(name = "getPlatformName")]
    #[view]
    pub fn get_platform_name(&self, platform_id: FixedBytes<32>) -> Result<String, Errors> {
        let ptr = self.platform_data.get(platform_id);
        Ok(ptr.name.get_string())
    }

    /// Get a platform origin
    #[selector(name = "getPlatformOrigin")]
    #[view]
    pub fn get_platform_origin(&self, platform_id: FixedBytes<32>) -> Result<String, Errors> {
        let ptr = self.platform_data.get(platform_id);
        Ok(ptr.origin.get_string())
    }
}
