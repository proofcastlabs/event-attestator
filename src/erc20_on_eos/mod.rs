//! # The `pERC20-on-EOS` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! native ERC20 tokens and the  pToken equivalent on the host EOS blockchain. This
//! core consists of two light clients that manage the state of the two chains, along
//! with the creation and signing of transactions related to each chain.
//!
//! __NOTE:__ All `debug_` prefixed functions can only be used if the core is
//! built with the `debug` feaure flag enabled in the `Cargo.toml`:
//!
//! ```no_compile
//! ptokens_core = { version = "1.0.0", features = ["debug"] }
//! ```
pub use crate::chains::eth::core_initialization::initialize_eth_enclave::{
    maybe_initialize_eth_enclave,
};

pub use crate::erc20_on_eos::{
    eth::submit_eth_block::submit_eth_block_to_core,
    eos::{
        submit_eos_block::submit_eos_block_to_core,
        initialize_eos_core::maybe_initialize_eos_core,
    },
};

pub mod eth;
pub mod eos;

pub(crate) mod check_core_is_initialized;
