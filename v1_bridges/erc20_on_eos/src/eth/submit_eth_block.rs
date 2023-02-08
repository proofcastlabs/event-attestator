use common::{
    chains::eth::{
        add_block_and_receipts_to_db::maybe_add_eth_block_and_receipts_to_db_and_return_state,
        check_parent_exists::check_for_parent_of_eth_block_in_state,
        eth_database_transactions::{
            end_eth_db_transaction_and_return_state,
            start_eth_db_transaction_and_return_state,
        },
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        increment_eos_account_nonce::maybe_increment_eos_account_nonce_and_return_state,
        remove_old_eth_tail_block::maybe_remove_old_eth_tail_block_and_return_state,
        remove_receipts_from_canon_block::maybe_remove_receipts_from_eth_canon_block_and_return_state,
        update_eth_canon_block_hash::maybe_update_eth_canon_block_hash_and_return_state,
        update_eth_linker_hash::maybe_update_eth_linker_hash_and_return_state,
        update_eth_tail_block_hash::maybe_update_eth_tail_block_hash_and_return_state,
        update_latest_block_hash::maybe_update_latest_eth_block_hash_and_return_state,
        validate_block_in_state::validate_eth_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
        EthState,
    },
    core_type::CoreType,
    dictionaries::eos_eth::get_eos_eth_token_dictionary_from_db_and_add_to_eth_state,
    traits::DatabaseInterface,
    types::Result,
};

use crate::eth::{
    account_for_fees::maybe_account_for_fees,
    divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_token_address,
    eos_tx_info::{
        filter_out_zero_value_eos_tx_infos_from_state,
        filter_submission_material_for_peg_in_events_in_state,
        maybe_parse_eos_tx_info_from_canon_block_and_add_to_state,
        maybe_sign_eos_txs_and_add_to_eth_state,
    },
    get_output_json::get_output_json,
};
/// # Submit ETH Block to Core
///
/// The main submission pipeline. Submitting an ETH block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ETH
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_eth_block_to_core<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting ETH block to enclave...");
    CoreType::check_is_initialized(db)
        .and_then(|_| parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(db)))
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_eth_block_in_state)
        .and_then(get_eos_eth_token_dictionary_from_db_and_add_to_eth_state)
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_eos_tx_info_from_canon_block_and_add_to_state)
        .and_then(maybe_account_for_fees)
        .and_then(filter_out_zero_value_eos_tx_infos_from_state)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(maybe_sign_eos_txs_and_add_to_eth_state)
        .and_then(maybe_increment_eos_account_nonce_and_return_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_output_json)
}
