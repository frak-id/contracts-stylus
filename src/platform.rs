use alloc::{string::String, vec, vec::Vec};
use core::marker::PhantomData;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, B128, U256},
    alloy_sol_types::sol,
    crypto::keccak,
    msg::{self},
    prelude::*,
    storage::{
        StorageAddress, StorageArray, StorageB256, StorageB32, StorageMap, StorageString,
        StorageU256,
    },
};

pub trait PlateformParams {}

// Define events and errors in the contract
sol! {
    error NotPlateformOwner();
    error PlateformAlreadyExist();
    error InvalidPubKey();
    error InvalidMetadata();
}

#[derive(SolidityError)]
pub enum PlateformError {
    NotPlateformOwner(NotPlateformOwner),
    PlateformAlreadyExist(PlateformAlreadyExist),
    InvalidPubKey(InvalidPubKey),
    InvalidMetadata(InvalidMetadata),
}

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
type PlatformMetadataType = (String, Address, FixedBytes<4>, FixedBytes<32>, [U256; 4]);

// Define the global conntract storage
#[solidity_storage]
pub struct PlateformContract<T: PlateformParams> {
    // The plateform metadata storage (plateform_id => PlatformMetadata)
    platform_data: StorageMap<B128, PlatformMetadata>,
    phantom: PhantomData<T>,
}

/// Internal method stuff
impl<T: PlateformParams> PlateformContract<T> {
    /// Check if a plateform exist
    fn _plateform_exist(&self, plateform_id: B128) -> bool {
        self.platform_data.get(plateform_id).owner.is_empty()
    }

    /// Create a new plateform
    /// returns the created plateform_id
    pub fn _create_plateform(
        &mut self,
        name: String,
        owner: Address,
        content_type: FixedBytes<4>,
        origin: FixedBytes<32>,
        public_key: [U256; 4],
    ) -> Result<B128, PlateformError> {
        let plateform_id = B128::from_slice(keccak(origin).as_slice());
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

        // Ensure the pub key is valid (no zero value)
        if public_key.iter().any(|x| x.is_zero()) {
            return Err(PlateformError::InvalidPubKey(InvalidPubKey {}));
        }

        // Get the right metadata setter
        let mut metadata = self.platform_data.setter(plateform_id);
        // Create the new plateform metadata
        metadata.name.set_str(name);
        metadata.owner.set(owner);
        metadata.content_type.set(content_type);
        metadata.origin.set(origin);
        metadata.public_key.setter(0).unwrap().set(public_key[0]);
        metadata.public_key.setter(1).unwrap().set(public_key[1]);
        metadata.public_key.setter(2).unwrap().set(public_key[2]);
        metadata.public_key.setter(3).unwrap().set(public_key[3]);

        // Return the created plateform_id
        Ok(plateform_id)
    }

    /// Update a plateform metadata (could be name or owner)
    fn _update_plateform_metadata(
        &mut self,
        plateform_id: B128,
        name: String,
        owner: Address,
    ) -> Result<(), PlateformError> {
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

    /// Update a plateform public key
    fn _update_plateform_pubclic_key(
        &mut self,
        plateform_id: B128,
        public_key: [U256; 4],
    ) -> Result<(), PlateformError> {
        // Ensure the pub key is valid (no zero value)
        // TODO: Maybe a more robust check against BLS curve could be done
        if public_key.iter().any(|x| x.is_zero()) {
            return Err(PlateformError::InvalidPubKey(InvalidPubKey {}));
        }

        // Get the right metadata setter
        let mut metadata = self.platform_data.setter(plateform_id);
        // Create the new plateform metadata
        metadata.public_key.setter(0).unwrap().set(public_key[0]);
        metadata.public_key.setter(1).unwrap().set(public_key[1]);
        metadata.public_key.setter(2).unwrap().set(public_key[2]);
        metadata.public_key.setter(3).unwrap().set(public_key[3]);
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
        plateform_id: B128,
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

    /// Update the plateform public key
    #[selector(name = "updatePlateformPublicKey")]
    pub fn update_public_key(
        &mut self,
        plateform_id: B128,
        public_key: [U256; 4],
    ) -> Result<(), PlateformError> {
        let caller = msg::sender();
        // Ensure the caller is the owner
        if !self.platform_data.get(plateform_id).owner.eq(&caller) {
            return Err(PlateformError::NotPlateformOwner(NotPlateformOwner {}));
        }

        // Update the plateform public key
        self._update_plateform_pubclic_key(plateform_id, public_key)
    }

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
            [
                ptr.public_key.get(0).unwrap(),
                ptr.public_key.get(1).unwrap(),
                ptr.public_key.get(2).unwrap(),
                ptr.public_key.get(3).unwrap(),
            ],
        ))
    }
}
