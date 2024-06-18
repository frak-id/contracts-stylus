// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_std, no_main)]
extern crate alloc;
extern crate core;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

use alloc::{string::String, vec::Vec};

use alloy_primitives::{keccak256, U64};
use alloy_sol_types::SolType;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    alloy_sol_types::sol,
    block,
    crypto::keccak,
    evm,
    prelude::*,
    storage::{StorageBool, StorageMap, StorageU256, StorageU64},
};

// Utility functions and helpers used across the library
mod errors;
mod platform;
mod utils;

use platform::{PlatformContract, PlatformParams};
use utils::{
    eip712::{Eip712, Eip712Params},
    owned::{Owned, OwnedParams},
};

use crate::errors::{Errors, InvalidPlatformSignature, Unauthorized};

struct CoreParam;

impl OwnedParams for CoreParam {}
impl PlatformParams for CoreParam {}
impl Eip712Params for CoreParam {
    // Static fields
    const NAME: &'static str = "ContentConsumption";
    const VERSION: &'static str = "0.0.1";
}

// Define events and errors in the contract
sol! {
    event CcuPushed(bytes32 indexed platformId, address user, uint256 totalConsumption);
}

/// Define the user consumption data on the given platform
#[solidity_storage]
pub struct UserConsumption {
    ccu: StorageU256,
    update_timestamp: StorageU64,
}

// Define the global contract storage
#[solidity_storage]
#[entrypoint]
pub struct ContentConsumptionContract {
    // The user activity storage (user => platform_id => UserConsumption)
    user_consumptions: StorageMap<Address, StorageMap<FixedBytes<32>, UserConsumption>>,
    // The consumption nonce for each user's
    consumption_nonce: StorageMap<FixedBytes<32>, StorageU256>,
    // All the allowed validator
    allowed_validators: StorageMap<Address, StorageBool>,
    // All the inherited contracts
    #[borrow]
    platform: PlatformContract<CoreParam>,
    #[borrow]
    eip712: Eip712<CoreParam>,
    #[borrow]
    owned: Owned<CoreParam>,
}

// Private helper methods
impl ContentConsumptionContract {
    /// Get the user to platform nonce key
    fn user_to_platform_nonce_key(user: Address, platform_id: FixedBytes<32>) -> FixedBytes<32> {
        let mut nonce_data = Vec::new();
        nonce_data.extend_from_slice(&user[..]);
        nonce_data.extend_from_slice(&platform_id[..]);
        keccak256(nonce_data)
    }
}

/// Declare that `ContentConsumptionContract` is a contract with the following external methods.
#[external]
#[inherit(PlatformContract<CoreParam>, Owned<CoreParam>, Eip712<CoreParam>)]
impl ContentConsumptionContract {
    /* -------------------------------------------------------------------------- */
    /*                                 Constructor                                */
    /* -------------------------------------------------------------------------- */

    /// Initialize the contract with an owner.
    /// TODO: No constructor possible atm, so going with init method called during contract creation via multicall
    /// See: https://github.com/OffchainLabs/stylus-sdk-rs/issues/99
    #[selector(name = "initialize")]
    pub fn initialize(&mut self, owner: Address) -> Result<(), Errors> {
        // Init the eip712 domain
        self.eip712.initialize();
        // Init the contract with the given owner
        self.owned.initialize(owner)?;
        // Set that the owner is an allowed validator
        self.allowed_validators.setter(owner).set(true);

        // Return the success
        Ok(())
    }

    /// Get the current nonce for the given platform
    #[selector(name = "getNonceForPlatform")]
    pub fn get_nonce_for_platform(&self, user: Address, platform_id: FixedBytes<32>) -> U256 {
        self.consumption_nonce
            .get(Self::user_to_platform_nonce_key(user, platform_id))
    }

    /* -------------------------------------------------------------------------- */
    /*                                  CCU push                                  */
    /* -------------------------------------------------------------------------- */

    /// Push a new consumption for a given platform
    #[selector(name = "pushCcu")]
    pub fn push_ccu(
        &mut self,
        user: Address,
        platform_id: FixedBytes<32>,
        added_consumption: U256,
        deadline: U256,
        v: u8,
        r: FixedBytes<32>,
        s: FixedBytes<32>,
    ) -> Result<(), Errors> {
        // No need to check that te platform exists, as the consumption will be rejected
        //  if the recovered address is zero, and if the owner doesn't match the recovered address

        // Get the user consumption nonce
        let nonce_key = Self::user_to_platform_nonce_key(user, platform_id);
        let current_nonce = self.consumption_nonce.get(nonce_key);

        // Rebuild the signed data
        let struct_hash = keccak(
            <sol! { (bytes32, address, bytes32, uint256, uint256, uint256) }>::encode(&(
                keccak(b"ValidateConsumption(address user,bytes32 platformId,uint256 addedConsumption,uint256 nonce,uint256 deadline)").0,
                user,
                platform_id.0,
                added_consumption,
                current_nonce,
                deadline,
            )),
        );

        // Do an ecdsa recovery check on the signature
        let recovered_address = self
            .eip712
            .recover_typed_data_signer(struct_hash, v, r, s)?;

        // Check if the recovered address is in the allowed validator
        if !self.allowed_validators.get(recovered_address) {
            // Check if the platform owner is the same
            let platform_owner = self.platform.get_platform_owner(platform_id);
            if platform_owner.is_zero() || platform_owner != recovered_address {
                return Err(Errors::InvalidPlatformSignature(
                    InvalidPlatformSignature {},
                ));
            }

            // Otherwise, add the platform owner as validator
            self.allowed_validators.setter(platform_owner).set(true);
        }

        // Increase current nonce
        self.consumption_nonce
            .setter(nonce_key)
            .set(current_nonce + U256::from(1));

        // Get the current state
        let mut storage_ptr = self.user_consumptions.setter(user);
        let mut storage_ptr = storage_ptr.setter(platform_id);
        let last_ccu = storage_ptr.ccu.get();

        // Get the current timestamp
        let current_timestamp = block::timestamp();

        let total_consumption = last_ccu + added_consumption;

        // Emit the event
        evm::log(CcuPushed {
            platformId: platform_id.0,
            user,
            totalConsumption: total_consumption,
        });

        // Update the ccu amount
        storage_ptr.ccu.set(total_consumption);
        // Update the update timestamp
        storage_ptr
            .update_timestamp
            .set(U64::from(current_timestamp));

        // Return the success
        Ok(())
    }

    /// Set a new allowed validator
    #[selector(name = "setAllowedValidator")]
    pub fn set_allowed_validator(
        &mut self,
        validator: Address,
        allowed: bool,
    ) -> Result<(), Errors> {
        // Ensure that the caller is the owner
        self.owned.only_owner()?;

        // Add the validator
        self.allowed_validators.setter(validator).set(allowed);

        // Return the success
        Ok(())
    }

    /* -------------------------------------------------------------------------- */
    /*                           Register new platform                            */
    /* -------------------------------------------------------------------------- */

    /// Register a new platform (only callable by the owner)
    #[selector(name = "registerPlatform")]
    pub fn register_platform(
        &mut self,
        name: String,
        origin: String,
        owner: Address,
        content_type: FixedBytes<32>,
        deadline: U256,
        v: u8,
        r: FixedBytes<32>,
        s: FixedBytes<32>,
    ) -> Result<FixedBytes<32>, Errors> {
        // Ensure that the caller is the owner
        // TODO: Idk why, but a call to `self.owned.only_owner()` is not working
        // TODO: msg::sender() return 0 on v0.5.0, can't compile on 0.4.3

        // Rebuild the signed data
        let struct_hash = keccak(
            <sol! { (bytes32, address, bytes32, bytes32, uint256) }>::encode(&(
                keccak(b"CreateNewPlatform(address owner,bytes32 name,bytes32 origin,uint256 deadline)").0,
                owner,
                keccak256(&name.as_bytes()).0,
                keccak256(&origin.as_bytes()).0,
                deadline,
            )),
        );

        // Do an ecdsa recovery check on the signature
        let recovered_address = self
            .eip712
            .recover_typed_data_signer(struct_hash, v, r, s)?;

        // Ensure it's the owner who signed the platform creation
        if recovered_address.is_zero() || recovered_address != self.owned.get_owner() {
            return Err(Errors::Unauthorized(Unauthorized {}));
        }

        // Create the platform
        let platform_id = self
            .platform
            .create_platform(name, origin, owner, content_type)?;

        // Return the success and the created platform id
        Ok(platform_id)
    }

    /* -------------------------------------------------------------------------- */
    /*                          Consumption read methods                          */
    /* -------------------------------------------------------------------------- */

    /// Get the user consumption on a content
    #[selector(name = "getUserConsumption")]
    #[view]
    pub fn get_user_consumption(
        &self,
        user: Address,
        platform_id: FixedBytes<32>,
    ) -> Result<(U256, U256), Errors> {
        // Get the ptr to the platform metadata
        let storage_ptr = self.user_consumptions.get(user);
        let storage_ptr = storage_ptr.get(platform_id);
        // Return every field we are interested in
        Ok((
            // CCU + update time
            storage_ptr.ccu.get(),
            U256::from(storage_ptr.update_timestamp.get()),
        ))
    }
}
