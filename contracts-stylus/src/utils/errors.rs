use stylus_sdk::{alloy_sol_types::sol, prelude::SolidityError};

// Define the global errors
sol! {
    error AlreadyInitialized();

    // Eip 712
    error EcRecoverError();

    error InvalidPlatformSignature();
}

#[derive(SolidityError)]
pub enum Errors {
    AlreadyInitialized(AlreadyInitialized),

    EcRecoverError(EcRecoverError),

    InvalidPlatformSignature(InvalidPlatformSignature),
}
