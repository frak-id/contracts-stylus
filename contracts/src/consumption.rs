use alloc::{vec, vec::Vec};
use core::marker::PhantomData;

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, U256, U64},
    block::{self},
    prelude::*,
    storage::{StorageMap, StorageU256, StorageU64},
};

use crate::errors::{Errors, TooCloseConsumption};

pub trait UserConsumptionParams {}

/// Define the user consumption data on the given platform
#[solidity_storage]
pub struct UserConsumption {
    ccu: StorageU256,
    update_timestamp: StorageU64,
}

// Define the global consumption contract
#[solidity_storage]
pub struct ConsumptionContract<T: UserConsumptionParams> {
    // The user activity storage (user => platform_id => UserConsumption)
    user_consumptions: StorageMap<Address, StorageMap<FixedBytes<32>, UserConsumption>>,
    phantom: PhantomData<T>,
}

/// Internal method stuff
impl<T: UserConsumptionParams> ConsumptionContract<T> {
    /// Update a user consumption by the given `added_consumption`
    pub fn update_user_consumption(
        &mut self,
        user: Address,
        platform_id: FixedBytes<32>,
        added_consumption: U256,
    ) -> Result<(), Errors> {
        // Get the current state
        let mut storage_ptr = self.user_consumptions.setter(user);
        let mut storage_ptr = storage_ptr.setter(platform_id);
        let last_update = storage_ptr.update_timestamp.get().to::<u64>();
        let last_ccu = storage_ptr.ccu.get();

        // Get the current timestamp
        let current_timestamp = block::timestamp();

        // If last update was less than one minute ago, abort
        if (last_update + 60) > current_timestamp {
            return Err(Errors::TooCloseConsumption(TooCloseConsumption {}));
        }

        // Update the ccu amount
        storage_ptr.ccu.set(last_ccu + added_consumption);
        // Update the update timestamp
        storage_ptr
            .update_timestamp
            .set(U64::from(current_timestamp));

        // Return a success
        Ok(())
    }
}

/// External method stuff
#[external]
impl<T: UserConsumptionParams> ConsumptionContract<T> {
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
    ) -> Result<(U256, U256), Vec<u8>> {
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
