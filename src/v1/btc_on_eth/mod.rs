//! # The `pBTC-on-ETH` pToken Core
//!
//! Here lies the functionality required for the cross-chain conversions between
//! native bitcoins and the `pBTC` pToken on the host ETH blockchain. This core
//! consists of two light clients that manage the state of the two chains, along
//! with the creation and signing of transactions related to each chain.

mod btc;
mod constants;
mod debug_functions;
mod eth;
mod get_enclave_state;
mod get_latest_block_numbers;
mod test_utils;
mod utils;

#[cfg(test)]
pub(crate) use self::btc::BtcOnEthEthTxInfo; // FIXME Current used in `chains::btc::btc_test_utils`
pub(crate) use self::{btc::BtcOnEthEthTxInfos, eth::BtcOnEthBtcTxInfos}; // FIXME Currently used in `State`.
pub use crate::{
    btc_on_eth::{
        btc::submit_btc_block_to_enclave,
        constants::CORE_TYPE,
        debug_functions::{
            debug_get_all_db_keys,
            debug_get_fee_withdrawal_tx,
            debug_get_signed_erc777_change_pnetwork_tx,
            debug_get_signed_erc777_proxy_change_pnetwork_by_proxy_tx,
            debug_get_signed_erc777_proxy_change_pnetwork_tx,
            debug_maybe_add_utxo_to_db,
            debug_mint_pbtc,
            debug_put_btc_on_eth_peg_in_basis_points_in_db,
            debug_put_btc_on_eth_peg_out_basis_points_in_db,
            debug_reprocess_btc_block,
            debug_reprocess_btc_block_with_fee_accrual,
            debug_reprocess_btc_block_with_nonce,
            debug_reprocess_eth_block,
            debug_reprocess_eth_block_with_fee_accrual,
            debug_set_accrued_fees,
        },
        eth::{maybe_add_erc777_contract_address, maybe_initialize_eth_enclave, submit_eth_block_to_enclave},
        get_enclave_state::get_enclave_state,
        get_latest_block_numbers::get_latest_block_numbers,
    },
    chains::{
        btc::{
            btc_debug_functions::{debug_set_btc_account_nonce, debug_set_btc_fee, debug_set_btc_utxo_nonce},
            core_initialization::initialize_btc_core::maybe_initialize_btc_core as maybe_initialize_btc_enclave,
            utxo_manager::{
                debug_utxo_utils::{
                    debug_add_multiple_utxos,
                    debug_clear_all_utxos,
                    debug_consolidate_utxos,
                    debug_consolidate_utxos_to_address,
                    debug_get_child_pays_for_parent_btc_tx,
                    debug_remove_utxo,
                },
                utxo_utils::get_all_utxos_as_json_string as get_all_utxos,
            },
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
    debug_functions::{
        debug_add_debug_signer,
        debug_add_multiple_debug_signers,
        debug_get_key_from_db,
        debug_remove_debug_signer,
        debug_set_key_in_db_to_value,
    },
};
