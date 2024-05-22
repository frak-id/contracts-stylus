//! Scripts for deploying and initializing the Frak smart contracts.

#![deny(clippy::missing_docs_in_private_items)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

pub mod cli;
pub mod commands;
pub mod constants;
pub mod errors;
pub mod utils;

/// Our build utils
pub mod build;

/// Our deploy utils
mod deploy;

// Our output utils
mod output_writer;

pub mod tx;
