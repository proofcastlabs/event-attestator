//! # The `pERC20-on-EVM` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! native ETH tokens and their ERC777 pToken equivalents on EVM compliant chains.
//! This core consists of two light clients that manage the state of the two chains,
//! along with the creation and signing of transactions related to each chain.
//!
//! __NOTE:__ All `debug_` prefixed functions can only be used if the core is
//! built with the `debug` feaure flag enabled in the `Cargo.toml`:
//!
//! ```no_compile
//! ptokens_core = { version = <version-here>, features = ["debug"] }
//! ```

pub(crate) mod check_core_is_initialized;
mod debug_functions;
pub(crate) mod eth;
pub(crate) mod fees_calculator;
pub(crate) mod get_all_db_keys;
pub(crate) mod get_enclave_state;
pub(crate) mod get_latest_block_numbers;
pub(crate) mod int;
pub(crate) mod test_utils;

pub use crate::{
    chains::eth::{
        eth_debug_functions::{
            debug_reset_eth_chain,
            debug_reset_evm_chain as debug_reset_int_chain,
            debug_set_eth_account_nonce,
            debug_set_eth_any_sender_nonce,
            debug_set_eth_gas_price,
            debug_set_evm_account_nonce as debug_set_int_account_nonce,
            debug_set_evm_any_sender_nonce as debug_set_int_any_sender_nonce,
            debug_set_evm_gas_price as debug_set_int_gas_price,
        },
        eth_message_signer::{
            sign_ascii_msg_with_eth_key_with_no_prefix,
            sign_ascii_msg_with_eth_key_with_prefix,
            sign_ascii_msg_with_evm_key_with_no_prefix as sign_ascii_msg_with_int_key_with_no_prefix,
            sign_ascii_msg_with_evm_key_with_prefix as sign_ascii_msg_with_int_key_with_prefix,
            sign_hex_msg_with_eth_key_with_prefix,
            sign_hex_msg_with_evm_key_with_prefix as sign_hex_msg_with_int_key_with_prefix,
        },
    },
    debug_mode::{debug_get_key_from_db, debug_set_key_in_db_to_value},
    erc20_on_int::{
        debug_functions::{
            debug_add_dictionary_entry,
            debug_get_add_supported_token_tx,
            debug_get_add_weth_unwrapper_address_tx,
            debug_get_remove_supported_token_tx,
            debug_remove_dictionary_entry,
            debug_reprocess_eth_block,
            debug_reprocess_eth_block_with_fee_accrual,
            debug_reprocess_eth_block_with_nonce,
            debug_reprocess_int_block,
            debug_reprocess_int_block_with_fee_accrual,
            debug_reprocess_int_block_with_nonce,
            debug_set_accrued_fees_in_dictionary,
            debug_set_fee_basis_points,
            debug_withdraw_fees_and_save_in_db,
        },
        eth::{initialize_eth_core::maybe_initialize_eth_core, submit_eth_block::submit_eth_block_to_core},
        get_all_db_keys::get_all_db_keys,
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
        int::{initialize_int_core::maybe_initialize_int_core, submit_int_block::submit_int_block_to_core},
    },
};
