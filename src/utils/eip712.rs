//! Helper to use eip712 separator on a given contract
use core::marker::PhantomData;
use stylus_sdk::{alloy_primitives::FixedBytes, block, contract, crypto::keccak, prelude::*};

pub trait Eip712Params {
    // Name of the contract
    const NAME: &'static str;
    const VERSION: &'static str;

    // For caching purposes
    const INITIAL_CHAIN_ID: u64;
    const INITIAL_DOMAIN_SEPARATOR: FixedBytes<32>;
}

sol_storage! {
    pub struct Eip712<T: Eip712Params> {
        PhantomData<T> phantom;
    }
}

impl<T: Eip712Params> Eip712<T> {
    /// Compute a new domain separator
    pub fn compute_domain_separator() -> Result<FixedBytes<32>, Vec<u8>> {
        let mut digest_input = [0u8; 160];
        digest_input[0..32].copy_from_slice(&keccak("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)".as_bytes())[..]);
        digest_input[32..64].copy_from_slice(&keccak(T::NAME.as_bytes())[..]);
        digest_input[64..96].copy_from_slice(&keccak(T::VERSION.as_bytes())[..]);
        digest_input[96..128].copy_from_slice(&block::chainid().to_be_bytes()[..]);
        digest_input[128..160].copy_from_slice(&contract::address()[..]);

        Ok(keccak(digest_input))
    }
}

#[external]
impl<T: Eip712Params> Eip712<T> {
    /// Get the current domain separator
    #[selector(name = "domainSeparator")]
    pub fn domain_separator(&mut self) -> Result<FixedBytes<32>, Vec<u8>> {
        if block::chainid() == T::INITIAL_CHAIN_ID {
            Ok(T::INITIAL_DOMAIN_SEPARATOR)
        } else {
            Ok(Eip712::<T>::compute_domain_separator()?)
        }
    }
}
