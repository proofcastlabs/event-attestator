//! # The `pEOS-on-INT` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! native EOS tokens and INT ERC777 pToken equivalents. This core consists of two
//! light clients that manage the state of the two chains, along with the creation
//! and signing of transactions related to each chain.

mod check_core_is_initialized;
mod constants;
mod debug_functions;
mod eos;
mod get_all_db_keys;
mod get_enclave_state;
mod get_latest_block_numbers;
mod int;
mod test_utils;

// FIXME Used in `State`
pub(crate) use self::{eos::EosOnIntIntTxInfos, int::EosOnIntEosTxInfos};
pub use crate::{
    chains::{
        eos::{
            core_initialization::initialize_eos_core::maybe_initialize_eos_core_with_eos_account_without_symbol as maybe_initialize_eos_core,
            eos_debug_functions::{
                debug_add_global_sequences_to_processed_list,
                debug_add_new_eos_schedule,
                debug_add_token_dictionary_entry,
                debug_disable_eos_protocol_feature,
                debug_enable_eos_protocol_feature,
                debug_remove_global_sequences_from_processed_list,
                debug_remove_token_dictionary_entry,
                debug_set_eos_account_nonce,
                debug_update_incremerkle,
            },
        },
        eth::{
            eth_debug_functions::{
                debug_reset_eth_chain as debug_reset_int_chain,
                debug_set_eth_account_nonce as debug_set_int_account_nonce,
                debug_set_eth_gas_price as debug_set_int_gas_price,
            },
            eth_message_signer::{
                sign_ascii_msg_with_eth_key_with_no_prefix as sign_ascii_msg_with_int_key_with_no_prefix,
                sign_ascii_msg_with_eth_key_with_prefix as sign_ascii_msg_with_int_key_with_prefix,
                sign_hex_msg_with_eth_key_with_prefix as sign_hex_msg_with_int_key_with_prefix,
            },
        },
    },
    debug_functions::{debug_get_key_from_db, debug_set_key_in_db_to_value},
    eos_on_int::{
        debug_functions::{debug_reprocess_eos_block, debug_reprocess_eos_block_with_nonce, debug_reprocess_int_block},
        eos::submit_eos_block_to_core,
        get_all_db_keys::get_all_db_keys,
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
        int::{maybe_initialize_int_core, submit_int_block_to_core},
    },
};
