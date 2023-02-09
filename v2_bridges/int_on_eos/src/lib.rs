//! # The `pINT-on-EOS` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! interim-chain tokens and the pToken equivalent on the host EOS-compatible blockchain.
//! This core consists of two light clients that manage the state of the two chains,
//! along with the creation and signing of transactions related to each chain.

mod constants;
mod debug_functions;
mod eos;
mod get_enclave_state;
mod get_latest_block_numbers;
mod int;
mod test_utils;

pub use common::chains::{
    eos::get_processed_actions_list::get_processed_actions_list,
    eth::eth_message_signer::{
        sign_ascii_msg_with_eth_key_with_no_prefix as sign_ascii_msg_with_int_key_with_no_prefix,
        sign_ascii_msg_with_eth_key_with_prefix as sign_ascii_msg_with_int_key_with_prefix,
        sign_hex_msg_with_eth_key_with_prefix as sign_hex_msg_with_int_key_with_prefix,
    },
};
pub use common_db::{debug_get_key_from_db, debug_set_key_in_db_to_value};
pub use common_debug_signers::{debug_add_debug_signer, debug_add_multiple_debug_signers, debug_remove_debug_signer};
pub use common_eos::{
    debug_add_global_sequences_to_processed_list,
    debug_add_new_eos_schedule,
    debug_add_token_dictionary_entry,
    debug_disable_eos_protocol_feature,
    debug_enable_eos_protocol_feature,
    debug_remove_global_sequences_from_processed_list,
    debug_remove_token_dictionary_entry,
    debug_set_eos_account_nonce,
    debug_update_incremerkle,
};
pub use common_eth::{
    debug_reset_eth_chain as debug_reset_int_chain,
    debug_set_eth_account_nonce as debug_set_int_account_nonce,
    debug_set_eth_gas_price as debug_set_int_gas_price,
};

pub use self::{
    constants::CORE_TYPE,
    debug_functions::{
        debug_get_add_supported_token_tx,
        debug_get_all_db_keys,
        debug_get_remove_supported_token_tx,
        debug_reprocess_eos_block,
        debug_reprocess_eos_block_with_nonce,
        debug_reprocess_int_block,
    },
    eos::{maybe_initialize_eos_core, submit_eos_block_to_core, IntOnEosIntTxInfos},
    get_enclave_state::get_enclave_state,
    get_latest_block_numbers::get_latest_block_numbers,
    int::{maybe_initialize_int_core, submit_int_block_to_core, submit_int_blocks_to_core, IntOnEosEosTxInfos},
};

#[macro_use]
extern crate common;
#[macro_use]
extern crate log;
#[macro_use]
extern crate paste;
