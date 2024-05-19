// Allow `cargo stylus export-abi` to generate a main function.
#![cfg_attr(not(feature = "export-abi"), no_main)]
extern crate alloc;

/// Use an efficient WASM allocator.
#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;
use alloc::vec;
use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256},
    alloy_sol_types::sol,
    crypto::keccak,
    prelude::*,
};

// Utility functions and helpers used across the library
mod consumption;
mod platform;
mod utils;

use consumption::{ConsumptionContract, UserConsumptionParams};
use platform::{PlateformContract, PlateformParams};
use utils::{
    eip712::{Eip712, Eip712Params},
    owned::{Owned, OwnedParams},
    signature::{EcRecoverTrait, PrecompileEcRecover},
};

// Define events and errors in the contract
sol! {
    error InvalidPlateformSignature();
}
#[derive(SolidityError)]
pub enum ContentConsumptionError {
    InvalidPlateformSignature(InvalidPlateformSignature),
}

struct CoreParam;

impl OwnedParams for CoreParam {}
impl PlateformParams for CoreParam {}
impl UserConsumptionParams for CoreParam {}
impl Eip712Params for CoreParam {
    // Static fields
    const NAME: &'static str = "ContentConsumption";
    const VERSION: &'static str = "0.0.1";

    // TODO: This fields should be computed, and place in the EIP712 storage, computed with an init method
    const INITIAL_CHAIN_ID: u64 = 1;
    const INITIAL_DOMAIN_SEPARATOR: FixedBytes<32> = FixedBytes::<32>([0; 32]);
}

// Define the global conntract storage
#[solidity_storage]
#[entrypoint]
pub struct ContentConsumptionContract {
    #[borrow]
    plateform: PlateformContract<CoreParam>,
    #[borrow]
    consumption: ConsumptionContract<CoreParam>,
    #[borrow]
    eip712: Eip712<CoreParam>,
    #[borrow]
    owned: Owned<CoreParam>,
}

// Private methods
impl ContentConsumptionContract {}

/// Declare that `ContentConsumptionContract` is a contract with the following external methods.
#[external]
#[inherit(PlateformContract<CoreParam>, ConsumptionContract<CoreParam>, Owned<CoreParam>, Eip712<CoreParam>)]
impl ContentConsumptionContract {
    /* -------------------------------------------------------------------------- */
    /*                                 Constructor                                */
    /* -------------------------------------------------------------------------- */

    /// Initialize the contract with an owner.
    /// TODO: No constructor poossible atm, so going with init method called during contract creation via multicall
    /// See: https://github.com/OffchainLabs/stylus-sdk-rs/issues/99
    #[selector(name = "initialize")]
    pub fn initialize(&mut self, owner: Address) -> Result<(), Vec<u8>> {
        // Init the contract with the given owner
        self.owned.initialize(owner)?;

        // Return the success
        Ok(())
    }

    /* -------------------------------------------------------------------------- */
    /*                                  CCU push                                  */
    /* -------------------------------------------------------------------------- */

    /// Push a new consumption for a given plateform
    pub fn push_ccu(
        &mut self,
        user: Address,
        plateform_id: FixedBytes<32>,
        added_consumption: U256,
        deadline: U256,
        v: u8,
        r: FixedBytes<32>,
        s: FixedBytes<32>,
    ) -> Result<(), Vec<u8>> {
        // Ensure that the caller is the owner of the plateform
        let plateform_owner = self.plateform.get_plateform_owner(plateform_id)?;

        // Rebuild the signed data
        // TODO: Should include more stuff here
        let mut struct_hash = [0u8; 192];
        struct_hash[0..32].copy_from_slice(&keccak(b"ValidateConsumption(address user,bytes32 plateformId,uint256 addedConsumption,uint256 deadline)")[..]);
        struct_hash[32..64].copy_from_slice(&user[..]);
        struct_hash[64..96].copy_from_slice(&plateform_id[..]);
        struct_hash[96..128].copy_from_slice(&added_consumption.to_be_bytes_vec()[..]);
        struct_hash[128..160].copy_from_slice(&deadline.to_be_bytes_vec()[..]);

        // Rebuild the digest input
        let mut digest_input = [0u8; 2 + 32 + 32];
        digest_input[0] = 0x19;
        digest_input[1] = 0x01;
        digest_input[2..34].copy_from_slice(&self.eip712.domain_separator()?[..]);
        digest_input[34..66].copy_from_slice(&keccak(struct_hash)[..]);

        // TODO: Perform a ecdsa signature verification from the owner
        let recovered_address = Address::from_slice(
            &PrecompileEcRecover::ecrecover(&keccak(digest_input), v, &r.0, &s.0).map_err(
                |_| {
                    ContentConsumptionError::InvalidPlateformSignature(InvalidPlateformSignature {})
                },
            )?,
        );

        // Ensure the plateform owner has signed the consumption
        if recovered_address.is_zero() || recovered_address != plateform_owner {
            return Err(ContentConsumptionError::InvalidPlateformSignature(
                InvalidPlateformSignature {},
            )
            .into());
        }

        // Push the new CCU
        self.consumption
            .update_user_consumption(user, plateform_id, added_consumption)?;

        // Return the success
        Ok(())
    }

    /* -------------------------------------------------------------------------- */
    /*                           Register new plateform                           */
    /* -------------------------------------------------------------------------- */

    /// Register a new plateform (only callable by the owner)
    #[selector(name = "registerPalteform")]
    pub fn register_plateform(
        &mut self,
        name: String,
        owner: Address,
        content_type: FixedBytes<4>,
        origin: FixedBytes<32>,
    ) -> Result<FixedBytes<32>, Vec<u8>> {
        // Ensure that the caller is the owner
        self.owned.only_owner()?;

        // Create the plateform
        let plateform_id = self
            .plateform
            ._create_plateform(name, owner, content_type, origin)?;

        // Return the success and the created plateform id
        Ok(plateform_id)
    }

    /*
     * TODO: -> Option to handle CCU
     *  -> Pushing new CCU should receive base CCU data (new amount), and check against a BLS signature if the owner of the plateform allowed it
     */
}
