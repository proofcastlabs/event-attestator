//! # The `pINT-on-ALGO` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! the pToken Interim Chain ERC20 tokens and their pToken counterparts on Algorand.
//! This core consists of two light clients that manage the state of the two chains,
//! along with the creation and signing of transactions related to each chain.
//!
//! __NOTE:__ All `debug_` prefixed functions can only be used if the core is
//! built with the `debug` feaure flag enabled in the `Cargo.toml`:
//!
//! ```no_compile
//! ptokens_core = { version = <version-here>, features = ["debug"] }
//! ```

pub(crate) mod algo;
pub(crate) mod check_core_is_initialized;
pub(crate) mod debug_functions;
pub(crate) mod get_all_db_keys;
pub(crate) mod get_enclave_state;
pub(crate) mod get_latest_block_numbers;
pub(crate) mod int;
pub(crate) mod test_utils;

pub use crate::{
    chains::{
        algo::{algo_debug_functions::debug_reset_algo_chain, algo_note_metadata::encode_algo_note_metadata},
        eth::eth_debug_functions::{
            debug_reset_eth_chain as debug_reset_int_chain,
            debug_set_eth_account_nonce as debug_set_int_account_nonce,
            debug_set_eth_gas_price as debug_set_int_gas_price,
        },
    },
    debug_mode::{debug_get_key_from_db, debug_set_key_in_db_to_value},
    int_on_algo::{
        algo::{initialize_algo_core::maybe_initialize_algo_core, submit_algo_block::submit_algo_block_to_core},
        debug_functions::{
            algo_block_reprocessor::{debug_reprocess_algo_block, debug_reprocess_algo_block_with_nonce},
            debug_add_dictionary_entry,
            debug_get_add_supported_token_tx,
            debug_get_algo_pay_tx,
            debug_remove_dictionary_entry,
            debug_set_algo_account_nonce,
            int_block_reprocessor::debug_reprocess_int_block,
            opt_in_to_application::debug_opt_in_to_application,
            opt_in_to_asset::debug_opt_in_to_asset,
        },
        get_all_db_keys::get_all_db_keys,
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
        int::{initialize_int_core::maybe_initialize_int_core, submit_int_block::submit_int_block_to_core},
    },
};
