#![recursion_limit = "256"] // NOTE: Because of the error macro.
#![allow(clippy::too_many_arguments)]

//! # The __`pToken`__ Core
//!
//! Herein lies the functionality required for the cross-chain conversions
//! between various blockchains allowing for decentalized swaps between a native
//! asset and a host chain's pTokenized version of that asset.
//!
//! __Note:__ When compiling the core, you may provide an optional environment
//! variable __`DB_KEY_PREFIX`__, which when used will prefix all database keys
//! with the provided argument. Via this, database key clashes can be avoided
//! if running multiple instances on one machine.

pub use crate::{
    core_type::CoreType,
    debug_functions::get_debug_signature_info,
    errors::AppError,
    traits::DatabaseInterface,
    types::{Bytes, Result},
    utils::get_core_version,
};

#[macro_use]
mod macros;
mod address;
mod chains;
mod constants;
mod core_type;
mod crypto_utils;
mod database_utils;
mod debug_functions;
mod dictionaries;
mod enclave_info;
mod errors;
mod fees;
mod metadata;
mod safe_addresses;
#[cfg(test)]
mod test_utils;
mod traits;
mod types;
mod utils;
mod v2;

pub mod btc_on_eos;
pub mod btc_on_eth;
pub mod eos_on_eth;
pub mod erc20_on_eos;
pub mod erc20_on_evm;

pub use v2::{btc_on_int, eos_on_int, erc20_on_int, int_on_algo, int_on_eos, int_on_evm};

#[cfg(test)]
extern crate simple_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate paste;
