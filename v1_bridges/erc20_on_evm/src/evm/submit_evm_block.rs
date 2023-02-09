
use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eth::{
    check_for_parent_of_evm_block_in_state,
    end_eth_db_transaction_and_return_state,
    maybe_add_evm_block_and_receipts_to_db_and_return_state,
    maybe_increment_eth_account_nonce_and_return_state,
    maybe_remove_old_evm_tail_block_and_return_state,
    maybe_remove_receipts_from_evm_canon_block_and_return_state,
    maybe_update_evm_canon_block_hash_and_return_state,
    maybe_update_evm_linker_hash_and_return_state,
    maybe_update_evm_tail_block_hash_and_return_state,
    maybe_update_latest_evm_block_hash_and_return_state,
    parse_eth_submission_material_and_put_in_state,
    start_eth_db_transaction_and_return_state,
    validate_evm_block_in_state,
    validate_receipts_in_state,
    EthState,
};

use crate::evm::{
    account_for_fees::maybe_account_for_fees,
    divert_to_safe_address::{
        maybe_divert_txs_to_safe_address_if_destination_is_token_address,
        maybe_divert_txs_to_safe_address_if_destination_is_vault_address,
    },
    eth_tx_info::{
        filter_out_zero_value_eth_tx_infos_from_state,
        filter_submission_material_for_redeem_events_in_state,
        maybe_parse_tx_info_from_canon_block_and_add_to_state,
        maybe_sign_eth_txs_and_add_to_evm_state,
    },
    get_evm_output_json::get_evm_output_json,
};

/// # Submit EVM Block to Core
///
/// The main submission pipeline. Submitting an ETH block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ETH
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_evm_block_to_core<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting EVM block to core...");
    CoreType::check_is_initialized(db)
        .and_then(|_| parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(db)))
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_evm_block_in_state)
        .and_then(|state| state.get_eth_evm_token_dictionary_and_add_to_state())
        .and_then(check_for_parent_of_evm_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_redeem_events_in_state)
        .and_then(maybe_add_evm_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_evm_block_hash_and_return_state)
        .and_then(maybe_update_evm_canon_block_hash_and_return_state)
        .and_then(maybe_update_evm_tail_block_hash_and_return_state)
        .and_then(maybe_update_evm_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_eth_tx_infos_from_state)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_vault_address)
        .and_then(maybe_sign_eth_txs_and_add_to_evm_state)
        .and_then(maybe_increment_eth_account_nonce_and_return_state)
        .and_then(maybe_remove_old_evm_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_evm_canon_block_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_evm_output_json)
}
