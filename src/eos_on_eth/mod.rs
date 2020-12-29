//! # The `pEOS-on-ETH` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! native EOS tokens and ETH ERC777 pToken equivalents. This core consists of two
//! light clients that manage the state of the two chains, along with the creation
//! and signing of transactions related to each chain.
//!
//! __NOTE:__ All `debug_` prefixed functions can only be used if the core is
//! built with the `debug` feaure flag enabled in the `Cargo.toml`:
//!
//! ```no_compile
//! ptokens_core = { version = "1.0.0", features = ["debug"] }
//! ```

pub mod debug_functions;
pub mod eos;
pub mod eth;
pub mod get_enclave_state;
pub mod get_latest_block_numbers;

pub(crate) mod check_core_is_initialized;

pub use crate::{
    chains::eos::core_initialization::initialize_eos_core::maybe_initialize_eos_core,
    eos_on_eth::{
        eth::initialize_eth_core::maybe_initialize_eth_core,
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
    },
};
