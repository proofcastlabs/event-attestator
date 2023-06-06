//! # The `pERC20-on-EVM` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! native ETH tokens and their ERC777 pToken equivalents on EVM compliant chains.
//! This core consists of two light clients that manage the state of the two chains,
//! along with the creation and signing of transactions related to each chain.

mod constants;
mod debug_functions;
mod evm;
mod fees_calculator;
mod get_enclave_state;
mod get_latest_block_numbers;
mod int;
mod test_utils;

pub use common_database_utils::{debug_get_key_from_db, debug_set_key_in_db_to_value};
pub use common_debug_signers::{debug_add_debug_signer, debug_add_multiple_debug_signers, debug_remove_debug_signer};
pub use common_eth::{
    sign_ascii_msg_with_eth_key_with_no_prefix as sign_ascii_msg_with_int_key_with_no_prefix,
    sign_ascii_msg_with_eth_key_with_prefix as sign_ascii_msg_with_int_key_with_prefix,
    sign_ascii_msg_with_evm_key_with_no_prefix,
    sign_ascii_msg_with_evm_key_with_prefix,
    sign_hex_msg_with_eth_key_with_prefix as sign_hex_msg_with_int_key_with_prefix,
    sign_hex_msg_with_evm_key_with_prefix,
};
pub use common_eth_debug::{
    debug_reset_eth_chain as debug_reset_int_chain,
    debug_reset_evm_chain,
    debug_set_eth_account_nonce as debug_set_int_account_nonce,
    debug_set_eth_any_sender_nonce as debug_set_int_any_sender_nonce,
    debug_set_eth_gas_price as debug_set_int_gas_price,
    debug_set_evm_account_nonce,
    debug_set_evm_any_sender_nonce,
    debug_set_evm_gas_price,
};

pub use self::{
    constants::CORE_TYPE,
    debug_functions::{
        debug_add_dictionary_entry,
        debug_get_add_supported_token_tx,
        debug_get_all_db_keys,
        debug_get_remove_supported_token_tx,
        debug_remove_dictionary_entry,
        debug_reprocess_evm_block,
        debug_reprocess_evm_block_with_fee_accrual,
        debug_reprocess_evm_block_with_nonce,
        debug_reprocess_int_block,
        debug_reprocess_int_block_with_fee_accrual,
        debug_reprocess_int_block_with_nonce,
        debug_set_accrued_fees_in_dictionary,
        debug_set_fee_basis_points,
        debug_withdraw_fees_and_save_in_db,
    },
    evm::{maybe_initialize_evm_core, submit_evm_block_to_core, submit_evm_blocks_to_core},
    get_enclave_state::get_enclave_state,
    get_latest_block_numbers::get_latest_block_numbers,
    int::{maybe_initialize_int_core, submit_int_block_to_core, submit_int_blocks_to_core},
};

#[macro_use]
extern crate common;
#[macro_use]
extern crate common_eth;
#[macro_use]
extern crate log;
#[macro_use]
extern crate paste;
#[macro_use]
extern crate lazy_static;
