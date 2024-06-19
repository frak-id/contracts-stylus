use alloy_primitives::{Address, FixedBytes, U256, U64};
use alloy_sol_types::{SolCall, SolType};
use openzeppelin_stylus::access::ownable::Ownable;
use stylus_sdk::{
    alloy_sol_types::sol,
    block,
    call::RawCall,
    crypto::keccak,
    evm, msg,
    prelude::*,
    storage::{StorageAddress, StorageMap, StorageU256, StorageU64},
};

use crate::utils::{
    eip712::{Eip712, Eip712Params},
    errors::{AlreadyInitialized, Errors, InvalidPlatformSignature},
    solidity::isAuthorizedCall,
};

// Define events and errors in the contract
sol! {
    event CcuPushed(bytes32 indexed channelId, address user, uint256 totalConsumption);
}

struct ConsumptionParam;

impl Eip712Params for ConsumptionParam {
    // Static fields
    const NAME: &'static str = "ChannelConsumption";
    const VERSION: &'static str = "0.0.1";
}

/// Define the user consumption data on the given platform
#[solidity_storage]
pub struct UserConsumption {
    ccu: StorageU256,
    update_timestamp: StorageU64,
}

/// Define the global contract storage
#[solidity_storage]
#[entrypoint]
pub struct ChannelConsumptionContract {
    // The user activity storage (user => channel_id => UserConsumption)
    user_consumptions: StorageMap<Address, StorageMap<FixedBytes<32>, UserConsumption>>,
    // Some general configurations
    frak_content_id: StorageU256,
    content_registry: StorageAddress,
    // The total tracked consumption
    total_consumption: StorageU256,
    // The ownable borrowing
    #[borrow]
    ownable: Ownable,
    #[borrow]
    eip712: Eip712<ConsumptionParam>,
}

/// Declare that `ContentConsumptionContract` is a contract with the following external methods.
#[external]
#[inherit(Ownable, Eip712<ConsumptionParam>)]
impl ChannelConsumptionContract {
    /* -------------------------------------------------------------------------- */
    /*                                 Constructor                                */
    /* -------------------------------------------------------------------------- */

    /// Initialize the contract with an owner.
    /// TODO: No constructor possible atm, so going with init method called during contract creation via multicall
    /// See: https://github.com/OffchainLabs/stylus-sdk-rs/issues/99
    #[selector(name = "initialize")]
    pub fn initialize(
        &mut self,
        owner: Address,
        frak_content_id: U256,
        content_registry: Address,
    ) -> Result<(), Errors> {
        // Ensure that the contract has not been initialized
        if !self.ownable.owner().is_zero() {
            return Err(Errors::AlreadyInitialized(AlreadyInitialized {}));
        }

        // Init our owner
        self.ownable._transfer_ownership(owner);

        // Init our global config
        self.frak_content_id.set(frak_content_id);
        self.content_registry.set(content_registry);

        // Return the success
        Ok(())
    }

    /* -------------------------------------------------------------------------- */
    /*                                  CCU push                                  */
    /* -------------------------------------------------------------------------- */

    /// Push a new consumption for a given platform
    #[selector(name = "pushCcu")]
    pub fn push_ccu(
        &mut self,
        channel_id: FixedBytes<32>,
        added_consumption: U256,
        deadline: U256,
        v: u8,
        r: FixedBytes<32>,
        s: FixedBytes<32>,
    ) -> Result<(), Errors> {
        // No need to check that te platform exists, as the consumption will be rejected
        //  if the recovered address is zero, and if the owner doesn't match the recovered address

        // Rebuild the signed data
        let user = msg::sender();
        let struct_hash = keccak(
            <sol! { (bytes32, address, bytes32, uint256, uint256) }>::encode(&(
                keccak(b"ValidateConsumption(address user,bytes32 channelId,uint256 addedConsumption,uint256 deadline)").0,
                user,
                channel_id.0,
                added_consumption,
                deadline,
            )),
        );

        // Do an ecdsa recovery check on the signature
        let recovered_address = self
            .eip712
            .recover_typed_data_signer(struct_hash, v, r, s)?;

        // Check if the validator has the validation roles
        let calldata = isAuthorizedCall {
            _contentId: self.frak_content_id.get(),
            _caller: recovered_address,
        }
        .encode();

        // Ensure the signer has the interaction validator roles for this content)
        let content_registry = self.content_registry.get();
        let has_all_roles_raw_result = RawCall::new_static()
            .call(content_registry, &calldata)
            .map_err(|_| Errors::InvalidPlatformSignature(InvalidPlatformSignature {}))?;
        let has_role = isAuthorizedCall::decode_returns(&has_all_roles_raw_result, false)
            .map_err(|_| Errors::InvalidPlatformSignature(InvalidPlatformSignature {}))?;

        // If it doesn't have the roles, return an error
        if !has_role._0 {
            return Err(Errors::InvalidPlatformSignature(
                InvalidPlatformSignature {},
            ));
        }

        // Get the current state
        let mut storage_ptr = self.user_consumptions.setter(user);
        let mut storage_ptr = storage_ptr.setter(channel_id);

        let total_consumption = storage_ptr.ccu.get() + added_consumption;

        // Emit the event
        evm::log(CcuPushed {
            channelId: channel_id.0,
            user,
            totalConsumption: total_consumption,
        });

        // Update the ccu amount
        storage_ptr.ccu.set(total_consumption);
        // Update the update timestamp
        storage_ptr
            .update_timestamp
            .set(U64::from(block::timestamp()));

        // Update the whole total consumption
        self.total_consumption
            .set(self.total_consumption.get() + added_consumption);

        // Return the success
        Ok(())
    }

    /// Get the consumption of a user on a given platform
    #[selector(name = "getUserConsumption")]
    pub fn get_user_consumption(
        &self,
        user: Address,
        channel_id: FixedBytes<32>,
    ) -> Result<U256, Errors> {
        // Get the user consumption
        let storage_ptr = self.user_consumptions.get(user);
        let storage_ptr = storage_ptr.get(channel_id);

        // Return the consumption
        Ok(storage_ptr.ccu.get())
    }

    /// Get the total consumption handled by the contract
    #[selector(name = "getTotalConsumption")]
    pub fn get_total_consumption(&self) -> Result<U256, Errors> {
        Ok(self.total_consumption.get())
    }
}
