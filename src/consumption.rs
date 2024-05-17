use alloc::{vec, vec::Vec};

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{Address, B128, U256, U64},
    alloy_sol_types::sol,
    block::{self},
    prelude::*,
    storage::{StorageMap, StorageU256, StorageU64},
};

// Define events and errors in the contract
sol! {
    error TooCloseConsumptiom();
}
#[derive(SolidityError)]
pub enum ConsumptionError {
    TooCloseConsumptiom(TooCloseConsumptiom),
}

/// Define the user consumption data on the given plateform
#[solidity_storage]
pub struct UserConsumption {
    ccu: StorageU256,
    update_timestamp: StorageU64,
}

/// Define the type of user consumption, in an ideal world, it should be a sol type
/// TODO: When alloy-rs is updated on the stylus SDK, use a sol! macro to define the type
type UserConsumptionType = (U256, U256);

// Define the global consumption contract
#[solidity_storage]
pub struct ConsumptionContract {
    // The user activity storage (user => plateform_id => UserConsumption)
    user_consumptions: StorageMap<Address, StorageMap<B128, UserConsumption>>,
}

/// Internal method stuff
impl ConsumptionContract {
    /// Update a user cosnumption by the given `added_consumption`
    fn _update_user_consumption(
        &mut self,
        user: Address,
        plateform_id: B128,
        added_consumption: U256,
    ) -> Result<(), ConsumptionError> {
        // Get the current state
        let mut user_consumption = self.user_consumptions.setter(user);
        let mut plateform_user_consumption = user_consumption.setter(plateform_id);
        let last_update = plateform_user_consumption
            .update_timestamp
            .get()
            .to::<u64>();
        let last_ccu = plateform_user_consumption.ccu.get();

        // Get the current timestamp
        let current_timestamp = block::timestamp();

        // If last update was less than one minute ago, abort
        if (last_update + 60) > current_timestamp {
            return Err(ConsumptionError::TooCloseConsumptiom(
                TooCloseConsumptiom {},
            ));
        }

        // Update the ccu amount
        plateform_user_consumption
            .ccu
            .set(last_ccu + added_consumption);
        // Update the update timestamp
        plateform_user_consumption
            .update_timestamp
            .set(U64::from(current_timestamp));

        // Return a success
        Ok(())
    }
}

/// External method stuff
#[external]
impl ConsumptionContract {
    /* -------------------------------------------------------------------------- */
    /*                          Consumption read methods                          */
    /* -------------------------------------------------------------------------- */

    /// Get the user consumption on a content
    #[selector(name = "getUserConsumption")]
    pub fn get_user_consumption(
        &self,
        user: Address,
        plateform_id: B128,
    ) -> Result<UserConsumptionType, Vec<u8>> {
        // Get the ptr to the plateform metadata
        let user_ptr = self.user_consumptions.get(user);
        let ptr = user_ptr.get(plateform_id);
        // Return every field we are interested in
        Ok((
            // CCU + update time
            ptr.ccu.get(),
            U256::from(ptr.update_timestamp.get()),
        ))
    }
}
