// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;
extern crate core;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;
use alloc::vec;

use alloy_primitives::keccak256;
use alloy_sol_types::SolType;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    alloy_sol_types::sol,
    crypto::keccak,
    prelude::*,
    storage::{StorageMap, StorageU256},
};

// Utility functions and helpers used across the library
mod consumption;
mod errors;
mod platform;
mod utils;

use consumption::{ConsumptionContract, UserConsumptionParams};
use platform::{PlatformContract, PlatformParams};
use utils::{
    eip712::{Eip712, Eip712Params},
    owned::{Owned, OwnedParams},
};

use crate::errors::{Errors, InvalidPlatformSignature, Unauthorized};

struct CoreParam;

impl OwnedParams for CoreParam {}
impl PlatformParams for CoreParam {}
impl UserConsumptionParams for CoreParam {}
impl Eip712Params for CoreParam {
    // Static fields
    const NAME: &'static str = "ContentConsumption";
    const VERSION: &'static str = "0.0.1";
}

// Define the global conntract storage
#[solidity_storage]
#[entrypoint]
pub struct ContentConsumptionContract {
    // All the inherited contracts
    #[borrow]
    platform: PlatformContract<CoreParam>,
    #[borrow]
    consumption: ConsumptionContract<CoreParam>,
    #[borrow]
    eip712: Eip712<CoreParam>,
    #[borrow]
    owned: Owned<CoreParam>,
    // The consumption nonce for each user's
    consumption_nonce: StorageMap<FixedBytes<32>, StorageU256>,
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
#[inherit(PlatformContract<CoreParam>, ConsumptionContract<CoreParam>, Owned<CoreParam>, Eip712<CoreParam>)]
impl ContentConsumptionContract {
    /* -------------------------------------------------------------------------- */
    /*                                 Constructor                                */
    /* -------------------------------------------------------------------------- */

    /// Initialize the contract with an owner.
    /// TODO: No constructor poossible atm, so going with init method called during contract creation via multicall
    /// See: https://github.com/OffchainLabs/stylus-sdk-rs/issues/99
    #[selector(name = "initialize")]
    pub fn initialize(&mut self, owner: Address) -> Result<(), Errors> {
        // Init the eip712 domain
        self.eip712.initialize();
        // Init the contract with the given owner
        self.owned.initialize(owner)?;

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

        // Ensure that the caller is the owner of the platform
        let platform_owner = self.platform.get_platform_owner(platform_id);

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

        // Ensure the platform owner has signed the consumption
        if recovered_address.is_zero() || recovered_address != platform_owner {
            return Err(Errors::InvalidPlatformSignature(
                InvalidPlatformSignature {},
            ));
        }

        // Increase current nonce
        self.consumption_nonce
            .setter(nonce_key)
            .set(current_nonce + U256::from(1));

        // Push the new CCU
        self.consumption
            .update_user_consumption(user, platform_id, added_consumption)?;

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
        content_type: FixedBytes<4>,
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

    /*
     * TODO: -> Option to handle CCU
     *  -> Pushing new CCU should receive base CCU data (new amount), and check against a BLS signature if the owner of the platform allowed it
     */
}
