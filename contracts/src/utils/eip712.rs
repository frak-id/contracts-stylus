//! Helper to use eip712 separator on a given contract
use core::marker::PhantomData;

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes, B256, U256, U64},
    alloy_sol_types::{sol, SolType},
    block, contract,
    crypto::keccak,
    prelude::*,
    storage::{StorageB256, StorageU64},
};

use super::signature::{EcRecoverTrait, PrecompileEcRecover};
use crate::errors::{EcRecoverError, Errors};

pub trait Eip712Params {
    // Name of the contract
    const NAME: &'static str;
    const VERSION: &'static str;
}

// Define the global owned contract storage
#[solidity_storage]
pub struct Eip712<T: Eip712Params> {
    cached_chain_id: StorageU64,
    cached_domain_separator: StorageB256,
    phantom: PhantomData<T>,
}

impl<T: Eip712Params> Eip712<T> {
    // Initialise the Eip712 contract (build initial cached domain separator)
    pub fn initialize(&mut self) {
        let initial_domain_separator = Eip712::<T>::compute_domain_separator();
        self.cached_chain_id.set(U64::from(block::chainid()));
        self.cached_domain_separator.set(initial_domain_separator);
    }

    /// Compute a new domain separator
    fn compute_domain_separator() -> B256 {
        // Direct rust variant, failing on contract address, idk why
        /*let mut digest_input = [0u8; 160];
        digest_input[0..32].copy_from_slice(&keccak("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)".as_bytes())[..]);
        digest_input[32..64].copy_from_slice(&keccak(T::NAME.as_bytes())[..]);
        digest_input[64..96].copy_from_slice(&keccak(T::VERSION.as_bytes())[..]);
        digest_input[96..128].copy_from_slice(&U256::from(block::chainid()).as_le_slice()[..]);
        digest_input[128..160].copy_from_slice(&contract::address()[..]);*/

        keccak(
            <sol! { (bytes32, bytes32, bytes32, uint256, address) }>::encode(&(
                keccak("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)").0,
                keccak(T::NAME.as_bytes()).0,
                keccak(T::VERSION.as_bytes()).0,
                U256::from(block::chainid()),
                contract::address(),
            )),
        )
    }

    /// Get the current domain separator
    /// Mutable since, if not cached, it could compute it and store it in cache
    pub fn domain_separator(&mut self) -> Result<B256, Errors> {
        // If the chain id is the same, return the cached domain separator
        if block::chainid() == self.cached_chain_id.get().to::<u64>() {
            Ok(self.cached_domain_separator.get())
        } else {
            // Otherwise, update it
            let domain_separator = Eip712::<T>::compute_domain_separator();
            // Updated cached infos
            self.cached_chain_id.set(U64::from(block::chainid()));
            self.cached_domain_separator.set(domain_separator);
            // And read it
            Ok(domain_separator)
        }
    }

    /// Recovery the typed data signer
    /// Mutable since, if domain separator not cached, it could recompute  it and store it in cache
    pub fn recover_typed_data_signer(
        &mut self,
        struct_hash: B256,
        v: u8,
        r: FixedBytes<32>,
        s: FixedBytes<32>,
    ) -> Result<Address, Errors> {
        // Rebuild the digest input
        let mut digest_input = [0u8; 2 + 32 + 32];
        digest_input[0] = 0x19;
        digest_input[1] = 0x01;
        digest_input[2..34].copy_from_slice(&self.domain_separator()?[..]);
        digest_input[34..66].copy_from_slice(&struct_hash[..]);

        // Do an ecdsa recovery check on the signature
        let recovered_address = Address::from_slice(
            &PrecompileEcRecover::ecrecover(&keccak(digest_input), v, &r.0, &s.0)
                .map_err(|_| Errors::EcRecoverError(EcRecoverError {}))?,
        );

        // Return the recovered address
        Ok(recovered_address)
    }
}

#[external]
impl<T: Eip712Params> Eip712<T> {
    /// Get the current domain separator
    #[selector(name = "domainSeparator")]
    pub fn read_domain_separator(&self) -> Result<FixedBytes<32>, Errors> {
        Ok(Eip712::<T>::compute_domain_separator())
    }
}
