use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eth::{
    check_for_parent_of_eth_block_in_state,
    end_eth_db_transaction_and_return_state,
    maybe_add_eth_block_and_receipts_to_db_and_return_state,
    maybe_remove_old_eth_tail_block_and_return_state,
    maybe_remove_receipts_from_eth_canon_block_and_return_state,
    maybe_update_eth_canon_block_hash_and_return_state,
    maybe_update_eth_linker_hash_and_return_state,
    maybe_update_eth_tail_block_hash_and_return_state,
    maybe_update_latest_eth_block_hash_and_return_state,
    parse_eth_submission_material_and_put_in_state,
    start_eth_db_transaction_and_return_state,
    validate_eth_block_in_state,
    validate_receipts_in_state,
    EthState,
};

use crate::eth::{
    account_for_fees::maybe_account_for_fees,
    btc_tx_info::maybe_parse_btc_tx_infos_and_add_to_state,
    create_btc_transactions::maybe_create_btc_txs_and_add_to_state,
    filter_receipts_in_state::filter_receipts_for_btc_on_eth_redeem_events_in_state,
    get_eth_output_json::get_eth_output_json,
    increment_btc_nonce::maybe_increment_btc_nonce_in_db_and_return_state,
};

/// # Submit ETH Block to Enclave
///
/// The main submission pipeline. Submitting an ETH block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ETH
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain a redeem event emitted by the smart-contract the enclave is watching, a BTC
/// transaction will be signed & returned to the caller.
pub fn submit_eth_block_to_enclave<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting ETH block to enclave...");
    CoreType::check_is_initialized(db)
        .and_then(|_| parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(db)))
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_eth_block_in_state)
        .and_then(check_for_parent_of_eth_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_receipts_for_btc_on_eth_redeem_events_in_state)
        .and_then(maybe_add_eth_block_and_receipts_to_db_and_return_state)
        .and_then(maybe_update_latest_eth_block_hash_and_return_state)
        .and_then(maybe_update_eth_canon_block_hash_and_return_state)
        .and_then(maybe_update_eth_tail_block_hash_and_return_state)
        .and_then(maybe_update_eth_linker_hash_and_return_state)
        .and_then(maybe_parse_btc_tx_infos_and_add_to_state)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_create_btc_txs_and_add_to_state)
        .and_then(maybe_increment_btc_nonce_in_db_and_return_state)
        .and_then(maybe_remove_old_eth_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_eth_output_json)
}
