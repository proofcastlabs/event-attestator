use common::{
    chains::btc::{
        add_btc_block_to_db::maybe_add_btc_block_to_db,
        btc_database_utils::{end_btc_db_transaction, start_btc_db_transaction},
        btc_submission_material::parse_submission_material_and_put_in_state,
        check_btc_parent_exists::check_for_parent_of_btc_block_in_state,
        deposit_address_info::validate_deposit_address_list_in_state,
        extract_utxos_from_p2sh_txs::maybe_extract_utxos_from_p2sh_txs_and_put_in_state,
        filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
        filter_utxos::filter_out_value_too_low_utxos_from_state,
        get_btc_block_in_db_format::create_btc_block_in_db_format_and_put_in_state,
        get_deposit_info_hash_map::get_deposit_info_hash_map_and_put_in_state,
        remove_old_btc_tail_block::maybe_remove_old_btc_tail_block,
        remove_tx_infos_from_canon_block::remove_tx_infos_from_canon_block_and_return_state,
        save_utxos_to_db::maybe_save_utxos_to_db,
        update_btc_canon_block_hash::maybe_update_btc_canon_block_hash,
        update_btc_latest_block_hash::maybe_update_btc_latest_block_hash,
        update_btc_linker_hash::maybe_update_btc_linker_hash,
        update_btc_tail_block_hash::maybe_update_btc_tail_block_hash,
        validate_btc_block_header::validate_btc_block_header_in_state,
        validate_btc_difficulty::validate_difficulty_of_btc_block_in_state,
        validate_btc_merkle_root::validate_btc_merkle_root,
        validate_btc_proof_of_work::validate_proof_of_work_of_btc_block_in_state,
    },
    core_type::CoreType,
    state::BtcState,
    traits::DatabaseInterface,
    types::Result,
};

use crate::btc::{
    account_for_fees::maybe_account_for_fees,
    divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_token_address,
    eos_tx_info::parse_eos_tx_infos_from_p2sh_deposits_and_add_to_state,
    get_btc_output_json::{create_btc_output_json_and_put_in_state, get_btc_output_as_string},
    increment_eos_nonce::maybe_increment_eos_nonce,
    sign_transactions::maybe_sign_canon_block_txs_and_add_to_state,
};

/// # Submit BTC Block to Core
///
/// The main submission pipeline. Submitting a BTC block to the core will - if that block is
/// valid & subsequent to the core's current latest block - advanced the piece of the BTC
/// blockchain held by the core in it's encrypted database. Should the submitted block
/// contain a deposit to an address derived from the core's BTC public key, an EOS
/// transaction will be signed & returned to the caller.
pub fn submit_btc_block_to_core<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting BTC block to core...");
    parse_submission_material_and_put_in_state(block_json_string, BtcState::init(db))
        .and_then(CoreType::check_core_is_initialized_and_return_btc_state)
        .and_then(start_btc_db_transaction)
        .and_then(check_for_parent_of_btc_block_in_state)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_difficulty_of_btc_block_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(validate_deposit_address_list_in_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(parse_eos_tx_infos_from_p2sh_deposits_and_add_to_state)
        .and_then(maybe_extract_utxos_from_p2sh_txs_and_put_in_state)
        .and_then(filter_out_value_too_low_utxos_from_state)
        .and_then(maybe_save_utxos_to_db)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(create_btc_block_in_db_format_and_put_in_state)
        .and_then(maybe_add_btc_block_to_db)
        .and_then(maybe_update_btc_latest_block_hash)
        .and_then(maybe_update_btc_canon_block_hash)
        .and_then(maybe_update_btc_tail_block_hash)
        .and_then(maybe_update_btc_linker_hash)
        .and_then(maybe_sign_canon_block_txs_and_add_to_state)
        .and_then(maybe_increment_eos_nonce)
        .and_then(maybe_remove_old_btc_tail_block)
        .and_then(create_btc_output_json_and_put_in_state)
        .and_then(remove_tx_infos_from_canon_block_and_return_state)
        .and_then(end_btc_db_transaction)
        .and_then(get_btc_output_as_string)
}
