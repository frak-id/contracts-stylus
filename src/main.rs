#![cfg_attr(not(feature = "export-abi"), no_main)]

#[cfg(feature = "export-abi")]
fn main() {
    frak_stylus::print_abi("GNU-GPLv3", "pragma solidity ^0.8.23;");
}