//! # The `pERC20-on-EOS` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! native ERC20 tokens and the  pToken equivalent on the host EOS blockchain. This
//! core consists of two light clients that manage the state of the two chains, along
//! with the creation and signing of transactions related to each chain.

mod add_vault_contract_address_to_db;
mod check_core_is_initialized;
mod constants;
mod debug_functions;
mod eos;
mod eth;
mod fees_calculator;
mod get_all_db_keys;
mod get_enclave_state;
mod get_latest_block_numbers;
mod test_utils;

// FIXME Used in `State`.
#[cfg(test)]
pub(in crate::erc20_on_eos) use self::eth::Erc20OnEosEosTxInfo;
pub(crate) use self::{eos::Erc20OnEosEthTxInfos, eth::Erc20OnEosEosTxInfos};
pub use crate::{
    chains::{
        eos::{
            core_initialization::initialize_eos_core::maybe_initialize_eos_core_without_eos_account_or_symbol as maybe_initialize_eos_core,
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
            get_processed_actions_list::get_processed_actions_list,
        },
        eth::{
            eth_debug_functions::{
                debug_reset_eth_chain,
                debug_set_eth_account_nonce,
                debug_set_eth_any_sender_nonce,
                debug_set_eth_gas_price,
            },
            eth_message_signer::{
                sign_ascii_msg_with_eth_key_with_no_prefix,
                sign_ascii_msg_with_eth_key_with_prefix,
                sign_hex_msg_with_eth_key_with_prefix,
            },
        },
    },
    debug_mode::{
        debug_add_debug_signer,
        debug_get_key_from_db,
        debug_remove_debug_signer,
        debug_set_key_in_db_to_value,
    },
    erc20_on_eos::{
        add_vault_contract_address_to_db::maybe_add_vault_contract_address_to_db,
        debug_functions::{
            debug_get_add_supported_token_tx,
            debug_get_erc20_vault_migration_tx as debug_get_perc20_migration_tx,
            debug_get_remove_supported_token_tx,
            debug_reprocess_eos_block,
            debug_reprocess_eos_block_with_fee_accrual,
            debug_reprocess_eos_block_with_nonce,
            debug_reprocess_eth_block,
            debug_reprocess_eth_block_with_fee_accrual,
            debug_set_accrued_fees_in_dictionary,
            debug_set_eos_fee_basis_points,
            debug_set_eth_fee_basis_points,
            debug_withdraw_fees_and_save_in_db,
        },
        eos::submit_eos_block_to_core,
        eth::{maybe_initialize_eth_core, submit_eth_block_to_core},
        get_all_db_keys::get_all_db_keys,
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
    },
};
