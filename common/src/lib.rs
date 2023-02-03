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

// FIXME Sort out the pub mods
#[macro_use]
pub mod macros;
pub mod address;
pub mod chains;
pub mod constants;
pub mod core_type;
pub mod crypto_utils;
pub mod database_utils;
pub mod debug_functions;
pub mod dictionaries;
pub mod enclave_info;
pub mod errors;
pub mod fees;
pub mod metadata;
pub mod safe_addresses;
pub mod state;
pub mod test_utils;
pub mod traits;
pub mod types;
pub mod utils;
pub mod v1;
pub mod v2;

pub use self::{
    v1::{btc_on_eos, btc_on_eth, eos_on_eth, erc20_on_eos, erc20_on_evm},
    v2::{int_on_eos, int_on_evm},
};

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
