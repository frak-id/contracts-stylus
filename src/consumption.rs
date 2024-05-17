use alloc::{vec, vec::Vec};

/// Import items from the SDK. The prelude contains common traits and macros.
use stylus_sdk::{
    alloy_primitives::{Address, B128, U256},
    prelude::*,
    storage::{StorageMap, StorageU256, StorageU64},
};

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

/// Declare that `ConsumptionContract` is a contract with the following external methods.
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
