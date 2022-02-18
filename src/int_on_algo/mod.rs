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
pub(crate) mod get_enclave_state;
pub(crate) mod get_latest_block_numbers;
pub(crate) mod int;
pub(crate) mod opt_in_to_asset;
pub(crate) mod test_utils;

pub use crate::{
    chains::{
        algo::{
            algo_note_metadata::encode_algo_note_metadata,
            core_initialization::reset_algo_chain::debug_reset_algo_chain,
        },
        eth::core_initialization::reset_eth_chain::debug_reset_eth_chain as debug_reset_int_chain,
    },
    int_on_algo::{
        algo::{initialize_algo_core::maybe_initialize_algo_core, submit_algo_block::submit_algo_block_to_core},
        debug_functions::{
            debug_add_dictionary_entry,
            debug_get_all_db_keys,
            debug_get_key_from_db,
            debug_remove_dictionary_entry,
            debug_set_algo_account_nonce,
            debug_set_key_in_db_to_value,
        },
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
        int::{initialize_int_core::maybe_initialize_int_core, submit_int_block::submit_int_block_to_core},
        opt_in_to_asset::opt_in_to_asset,
    },
};
/*
pub(crate) mod debug_functions;
pub(crate) mod evm;
pub(crate) mod fees_calculator;
pub(crate) mod get_enclave_state;
pub(crate) mod get_latest_block_numbers;
pub(crate) mod int;
pub(crate) mod test_utils;

pub use crate::{
    chains::eth::{
        core_initialization::reset_eth_chain::{debug_reset_eth_chain as debug_reset_int_chain, debug_reset_evm_chain},
        eth_debug_functions::{
            debug_set_eth_account_nonce as debug_set_int_account_nonce,
            debug_set_eth_any_sender_nonce as debug_set_int_any_sender_nonce,
            debug_set_evm_account_nonce,
            debug_set_evm_any_sender_nonce,
        },
        eth_message_signer::{
            sign_ascii_msg_with_eth_key_with_no_prefix as sign_ascii_msg_with_int_key_with_no_prefix,
            sign_ascii_msg_with_eth_key_with_prefix as sign_ascii_msg_with_int_key_with_prefix,
            sign_ascii_msg_with_evm_key_with_no_prefix,
            sign_ascii_msg_with_evm_key_with_prefix,
            sign_hex_msg_with_eth_key_with_prefix as sign_hex_msg_with_int_key_with_prefix,
            sign_hex_msg_with_evm_key_with_prefix,
        },
    },
    int_on_evm::{
        debug_functions::{
            block_reprocessors::{
                debug_reprocess_evm_block,
                debug_reprocess_evm_block_with_fee_accrual,
                debug_reprocess_evm_block_with_nonce,
                debug_reprocess_int_block,
                debug_reprocess_int_block_with_fee_accrual,
                debug_reprocess_int_block_with_nonce,
            },
            debug_add_dictionary_entry,
            debug_get_add_supported_token_tx,
            debug_get_all_db_keys,
            debug_get_key_from_db,
            debug_get_remove_supported_token_tx,
            debug_remove_dictionary_entry,
            debug_set_accrued_fees_in_dictionary,
            debug_set_evm_gas_price,
            debug_set_fee_basis_points,
            debug_set_int_gas_price,
            debug_set_key_in_db_to_value,
            debug_withdraw_fees_and_save_in_db,
        },
        evm::{initialize_evm_core::maybe_initialize_evm_core, submit_evm_block::submit_evm_block_to_core},
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
        int::{initialize_int_core::maybe_initialize_int_core, submit_int_block::submit_int_block_to_core},
    },
};
*/
