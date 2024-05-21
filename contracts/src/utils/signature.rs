//! From: https://github.com/cygaar/inkmate/blob/main/contracts/src/utils/ecrecover.rs
//! Waiting for PR: https://github.com/cygaar/inkmate/pull/12

// Re-export the EcRecoverTrait and implement it for the PrecompileEcRecover struct
pub use inkmate_common::crypto::ecrecover::EcRecoverTrait;
use inkmate_common::crypto::ecrecover::{
    EcdsaError, EC_RECOVER_ADDRESS_LAST_BYTE, EC_RECOVER_INPUT_LEN, NUM_BYTES_ADDRESS,
    NUM_BYTES_U256,
};
use stylus_sdk::{alloy_primitives::Address, call::RawCall};

pub struct PrecompileEcRecover;

impl EcRecoverTrait for PrecompileEcRecover {
    /// Calls the ecrecover EVM precompile through a static call
    fn ecrecover_implementation(
        input: [u8; EC_RECOVER_INPUT_LEN],
    ) -> Result<[u8; NUM_BYTES_ADDRESS], EcdsaError> {
        let res = RawCall::new_static()
            // Only get the last 20 bytes of the 32-byte return data
            .limit_return_data(NUM_BYTES_U256 - NUM_BYTES_ADDRESS, NUM_BYTES_ADDRESS)
            .call(
                Address::with_last_byte(EC_RECOVER_ADDRESS_LAST_BYTE),
                &input,
            )
            .map_err(|_| EcdsaError)?;

        res.try_into().map_err(|_| EcdsaError)
    }
}
