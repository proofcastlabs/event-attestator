//! # The `pBTC-on-INT` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! native bitcoins and the `pBTC` pToken on the host INT blockchain. This core
//! consists of two light clients that manage the state of the two chains, along
//! with the creation and signing of transactions related to each chain.
//!
//! __NOTE:__ All `debug_` prefixed functions can only be used if the core is
//! built with the `debug` feaure flag enabled in the `Cargo.toml`:
//!
//! ```no_compile
//! ptokens_core = { version = "1.0.0", features = ["debug"] }
//! ```

pub use crate::{
    btc_on_int::{
        btc::submit_btc_block::submit_btc_block_to_core,
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
        int::{initialize_int_core::maybe_initialize_int_enclave, submit_int_block::submit_int_block_to_core},
    },
    chains::{
        btc::{
            btc_debug_functions::{debug_set_btc_account_nonce, debug_set_btc_utxo_nonce},
            core_initialization::initialize_btc_core::maybe_initialize_btc_core,
        },
        eth::{
            core_initialization::reset_eth_chain::debug_reset_eth_chain as debug_reset_int_chain,
            eth_debug_functions::debug_set_eth_account_nonce as debug_set_int_account_nonce,
            eth_message_signer::{
                sign_ascii_msg_with_eth_key_with_no_prefix as sign_ascii_msg_with_int_key_with_no_prefix,
                sign_ascii_msg_with_eth_key_with_prefix as sign_ascii_msg_with_int_key_with_prefix,
                sign_hex_msg_with_eth_key_with_prefix as sign_hex_msg_with_int_key_with_prefix,
            },
        },
    },
};

//pub mod debug_functions; // FIXME
pub mod btc;
pub mod get_enclave_state;
pub mod get_latest_block_numbers;
pub mod int;

mod check_core_is_initialized;

pub(crate) mod test_utils;
