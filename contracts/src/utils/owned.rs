//! From: https://github.com/cairoeth/rustmate/blob/main/src/auth/owned.rs
//! Slight adaptation to use latest stylus version, and to have better errors

use core::marker::PhantomData;

use stylus_sdk::{
    alloy_primitives::Address, alloy_sol_types::sol, evm, msg, prelude::*, storage::StorageAddress,
};

use crate::errors::{AlreadyInitialized, Errors, InvalidInitialize, Unauthorized};

pub trait OwnedParams {}

// Declare events and Solidity error types
sol! {
    event OwnershipTransferred(address indexed owner);
}

// Define the global owned contract storage
#[solidity_storage]
pub struct Owned<T: OwnedParams> {
    // The owner of the contract
    owner: StorageAddress,
    phantom: PhantomData<T>,
}

impl<T: OwnedParams> Owned<T> {
    /// Initialize the contract with an owner.
    pub fn initialize(&mut self, _owner: Address) -> Result<(), Errors> {
        // Ensure that the contract has not been initialized
        if !self.owner.get().is_zero() {
            return Err(Errors::AlreadyInitialized(AlreadyInitialized {}));
        }

        // Ensure that the owner is not zero
        if _owner.is_zero() {
            return Err(Errors::InvalidInitialize(InvalidInitialize {}));
        }

        // Set the owner
        self.owner.set(_owner);

        // Emit the transfer
        evm::log(OwnershipTransferred { owner: _owner });

        Ok(())
    }

    /// Get the current owner of the contract
    pub fn get_owner(&self) -> Address {
        self.owner.get()
    }

    /// Ensure that the caller is the owner.
    pub fn only_owner(&self) -> Result<(), Errors> {
        if msg::sender() != self.owner.get() {
            Err(Errors::Unauthorized(Unauthorized {}))
        } else {
            Ok(())
        }
    }
}

/// All the owned external methods.
#[external]
impl<T: OwnedParams> Owned<T> {
    /// Transfer ownership of the contract to a new address.
    /// TODO: Handshake transfer via 2 signed eip712 message????
    #[selector(name = "transferOwnership")]
    pub fn transfer_ownership(&mut self, new_owner: Address) -> Result<(), Errors> {
        // Ensure that the owner is calling
        self.only_owner()?;

        // Set the new owner
        self.owner.set(new_owner);

        // Emit a ownership transfer event
        evm::log(OwnershipTransferred { owner: new_owner });

        Ok(())
    }
}
