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

pub trait PlateformParams {}

// Define events and errors in the contract
sol! {
    error NotPlateformOwner();
    error PlateformAlreadyExist();
    error InexistantPlateform();
    error InvalidMetadata();
}

#[derive(SolidityError)]
pub enum PlateformError {
    NotPlateformOwner(NotPlateformOwner),
    PlateformAlreadyExist(PlateformAlreadyExist),
    InexistantPlateform(InexistantPlateform),
    InvalidMetadata(InvalidMetadata),
}

/// Define the palteform metadata
#[solidity_storage]
pub struct PlatformMetadata {
    name: StorageString,
    owner: StorageAddress,
    content_type: StorageB32,
    origin: StorageB256,
}

/// Define the type of the plateform metadata, in an ideal world, it should be a sol type
/// TODO: When alloy-rs is updated on the stylus SDK, use a sol! macro to define the type
type PlatformMetadataType = (String, Address, FixedBytes<4>, FixedBytes<32>);

// Define the global conntract storage
#[solidity_storage]
pub struct PlateformContract<T: PlateformParams> {
    // The plateform metadata storage (plateform_id => PlatformMetadata)
    platform_data: StorageMap<FixedBytes<32>, PlatformMetadata>,
    phantom: PhantomData<T>,
}

/// Internal method stuff
impl<T: PlateformParams> PlateformContract<T> {
    /* -------------------------------------------------------------------------- */
    /*                                   Helpers                                  */
    /* -------------------------------------------------------------------------- */

    /// Check if a plateform exist
    fn _plateform_exist(&self, plateform_id: FixedBytes<32>) -> bool {
        self.platform_data.get(plateform_id).owner.is_empty()
    }

    /// Only allow the call for an existing plateform
    pub fn only_existing_plateform(
        &self,
        plateform_id: FixedBytes<32>,
    ) -> Result<(), PlateformError> {
        if !self._plateform_exist(plateform_id) {
            return Err(PlateformError::InexistantPlateform(InexistantPlateform {}));
        }
        Ok(())
    }

    /// Get the plateform owner
    pub fn get_plateform_owner(
        &self,
        plateform_id: FixedBytes<32>,
    ) -> Result<Address, PlateformError> {
        self.only_existing_plateform(plateform_id)?;
        Ok(self.platform_data.get(plateform_id).owner.get())
    }

    /* -------------------------------------------------------------------------- */
    /*                         Palteform creation / update                        */
    /* -------------------------------------------------------------------------- */

    /// Create a new plateform
    /// returns the created plateform_id
    pub fn _create_plateform(
        &mut self,
        name: String,
        owner: Address,
        content_type: FixedBytes<4>,
        origin: FixedBytes<32>,
    ) -> Result<FixedBytes<32>, PlateformError> {
        let plateform_id = keccak(origin);
        // Ensure the plateform doesn't already exist
        if self._plateform_exist(plateform_id) {
            return Err(PlateformError::PlateformAlreadyExist(
                PlateformAlreadyExist {},
            ));
        }

        // Ensure name and owner aren't empty
        if name.is_empty() | owner.is_zero() {
            return Err(PlateformError::InvalidMetadata(InvalidMetadata {}));
        }

        // Get the right metadata setter
        let mut metadata = self.platform_data.setter(plateform_id);
        // Create the new plateform metadata
        metadata.name.set_str(name);
        metadata.owner.set(owner);
        metadata.content_type.set(content_type);
        metadata.origin.set(origin);

        // Return the created plateform_id
        Ok(plateform_id)
    }

    /// Update a plateform metadata (could be name or owner)
    fn _update_plateform_metadata(
        &mut self,
        plateform_id: FixedBytes<32>,
        name: String,
        owner: Address,
    ) -> Result<(), PlateformError> {
        self.only_existing_plateform(plateform_id)?;

        // Ensure name isn't empty
        if name.is_empty() | owner.is_zero() {
            return Err(PlateformError::InvalidMetadata(InvalidMetadata {}));
        }
        // Get the right metadata setter
        let mut metadata = self.platform_data.setter(plateform_id);
        // Create the new plateform metadata
        metadata.name.set_str(name);
        metadata.owner.set(owner);
        // Return success
        Ok(())
    }
}

/// External method stuff
#[external]
impl<T: PlateformParams> PlateformContract<T> {
    /* -------------------------------------------------------------------------- */
    /*                          Plateform update methods                          */
    /* -------------------------------------------------------------------------- */

    /// Update the plateform metadatas
    #[selector(name = "updatePlateformMetadatas")]
    pub fn update_metadatas(
        &mut self,
        plateform_id: FixedBytes<32>,
        name: String,
        owner: Address,
    ) -> Result<(), PlateformError> {
        let caller = msg::sender();
        // Ensure the caller is the owner
        if !self.platform_data.get(plateform_id).owner.get().eq(&caller) {
            return Err(PlateformError::NotPlateformOwner(NotPlateformOwner {}));
        }

        // Update the plateform metadatas
        self._update_plateform_metadata(plateform_id, name, owner)
    }

    /* -------------------------------------------------------------------------- */
    /*                           Plateform read methods                           */
    /* -------------------------------------------------------------------------- */

    /// Get a plateform metadatas
    #[selector(name = "getPlateformMetadatas")]
    #[view]
    pub fn get_plateform_metadatas(
        &self,
        plateform_id: FixedBytes<32>,
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
        ))
    }
}
