use stylus_sdk::{alloy_sol_types::sol, prelude::SolidityError};

// Define the global errors
sol! {
    // Owned related
    error Unauthorized();
    error AlreadyInitialized();
    error InvalidInitialize();

    // Eip 712
    error EcRecoverError();

    // Platform related
    error NotPlatformOwner();
    error PlatformAlreadyExist();
    error PlatformDontExist();
    error InvalidMetadata();
    error InvalidPlatformSignature();
}

#[derive(SolidityError)]
pub enum Errors {
    Unauthorized(Unauthorized),
    AlreadyInitialized(AlreadyInitialized),
    InvalidInitialize(InvalidInitialize),

    EcRecoverError(EcRecoverError),

    NotPlatformOwner(NotPlatformOwner),
    PlatformAlreadyExist(PlatformAlreadyExist),
    PlatformDontExist(PlatformDontExist),
    InvalidMetadata(InvalidMetadata),
    InvalidPlatformSignature(InvalidPlatformSignature),
}
