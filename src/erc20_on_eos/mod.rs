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
//! ptokens_core = { version = "3.1.0", features = ["debug"] }
//! ```
pub use crate::{
    chains::{
        eos::{
            core_initialization::initialize_eos_core::maybe_initialize_eos_core_without_eos_account_or_symbol as maybe_initialize_eos_core,
            disable_protocol_feature::debug_disable_eos_protocol_feature,
            enable_protocol_feature::debug_enable_eos_protocol_feature,
            eos_debug_functions::{
                debug_add_global_sequences_to_processed_list,
                debug_add_new_eos_schedule,
                debug_add_token_dictionary_entry,
                debug_remove_global_sequences_from_processed_list,
                debug_remove_token_dictionary_entry,
                debug_set_eos_account_nonce,
                debug_update_incremerkle,
            },
            get_processed_actions_list::get_processed_actions_list,
        },
        eth::{
            core_initialization::reset_eth_chain::debug_reset_eth_chain,
            eth_debug_functions::{
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
    erc20_on_eos::{
        add_vault_contract_address_to_db::maybe_add_vault_contract_address_to_db,
        debug_functions::{
            debug_get_add_supported_token_tx,
            debug_get_all_db_keys,
            debug_get_erc20_vault_migration_tx as debug_get_perc20_migration_tx,
            debug_get_key_from_db,
            debug_get_remove_supported_token_tx,
            debug_set_accrued_fees_in_dictionary,
            debug_set_eos_fee_basis_points,
            debug_set_eth_fee_basis_points,
            debug_set_key_in_db_to_value,
            debug_withdraw_fees_and_save_in_db,
            eos_block_reprocessor::{
                debug_reprocess_eos_block,
                debug_reprocess_eos_block_with_fee_accrual,
                debug_reprocess_eos_block_with_nonce,
            },
            eth_block_reprocessor::{debug_reprocess_eth_block, debug_reprocess_eth_block_with_fee_accrual},
        },
        eos::submit_eos_block::submit_eos_block_to_core,
        eth::{
            initialize_eth_core::maybe_initialize_eth_enclave as maybe_initialize_eth_core,
            submit_eth_block::submit_eth_block_to_core,
        },
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
    },
};

pub mod debug_functions;
pub mod eos;
pub mod eth;
pub mod get_enclave_state;
pub mod get_latest_block_numbers;

pub(crate) mod add_vault_contract_address_to_db;
pub(crate) mod check_core_is_initialized;
pub(crate) mod fees_calculator;
pub(crate) mod test_utils;
