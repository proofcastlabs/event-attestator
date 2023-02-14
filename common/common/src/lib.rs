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
    algo_chain_id::AlgoChainId,
    btc_chain_id::BtcChainId,
    core_type::CoreType,
    eos_chain_id::EosChainId,
    eos_metadata::EosMetadata,
    errors::AppError,
    eth_chain_id::EthChainId,
    traits::DatabaseInterface,
    types::{Bytes, Result},
    utils::get_core_version,
};

// FIXME Sort out the pub mods
#[macro_use]
pub mod macros;
pub mod address;
pub mod algo_chain_id; // FIXME Ideally factor these out
mod btc_chain_id; // FIXME Ideally factor these out
pub mod constants;
pub mod core_type;
pub mod crypto_utils;
pub mod database_utils;
pub mod dictionaries;
mod eos_chain_id;
mod eos_metadata; // FIXME Move to EOS once cylic deps are sorted
pub mod errors;
mod eth_chain_id; // FIXME Ideally factor these out
pub mod metadata;
pub mod safe_addresses;
pub mod test_utils;
pub mod traits;
pub mod types;
pub mod utils;

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
