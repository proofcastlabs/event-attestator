use common::{
    chains::eos::{
        add_schedule::maybe_add_new_eos_schedule_to_db_and_return_state,
        append_interim_block_ids::append_interim_block_ids_to_incremerkle_in_state,
        eos_database_transactions::end_eos_db_transaction_and_return_state,
        eos_global_sequences::{
            get_processed_global_sequences_and_add_to_state,
            maybe_add_global_sequences_to_processed_list_and_return_state,
        },
        eos_submission_material::parse_submission_material_and_add_to_state,
        filter_action_proofs::{
            maybe_filter_duplicate_proofs_from_state,
            maybe_filter_out_action_proof_receipt_mismatches_and_return_state,
            maybe_filter_out_invalid_action_receipt_digests,
            maybe_filter_out_proofs_for_wrong_eos_account_name,
            maybe_filter_out_proofs_with_invalid_merkle_proofs,
            maybe_filter_out_proofs_with_wrong_action_mroot,
            maybe_filter_proofs_for_v1_redeem_actions,
        },
        get_active_schedule::get_active_schedule_from_db_and_add_to_state,
        get_enabled_protocol_features::get_enabled_protocol_features_and_add_to_state,
        get_eos_incremerkle::get_incremerkle_and_add_to_state,
        save_incremerkle::save_incremerkle_from_state_to_db,
        save_latest_block_id::save_latest_block_id_to_db,
        save_latest_block_num::save_latest_block_num_to_db,
        validate_producer_slot::validate_producer_slot_of_block_in_state,
        validate_signature::validate_block_header_signature,
        EosState,
    },
    traits::DatabaseInterface,
    types::Result,
    CoreType,
};

use crate::eos::{
    account_for_fees::maybe_account_for_fees,
    btc_tx_info::{
        maybe_filter_out_already_processed_tx_ids_from_state,
        maybe_filter_value_too_low_btc_tx_infos_in_state,
        maybe_parse_btc_tx_infos_and_put_in_state,
    },
    extract_utxos_from_btc_txs::maybe_extract_btc_utxo_from_btc_tx_in_state,
    filter_btc_txs_in_state::maybe_filter_btc_txs_in_state,
    get_eos_output::get_eos_output,
    maybe_increment_btc_signature_nonce_and_return_eos_state,
    save_btc_utxos_to_db::maybe_save_btc_utxos_to_db,
    sign_transactions::maybe_sign_txs_and_add_to_state,
};

pub fn submit_eos_block_to_core<D: DatabaseInterface>(db: &D, block_json: &str) -> Result<String> {
    info!("✔ Submitting EOS block to core...");
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| parse_submission_material_and_add_to_state(block_json, EosState::init(db)))
        .and_then(get_enabled_protocol_features_and_add_to_state)
        .and_then(get_incremerkle_and_add_to_state)
        .and_then(append_interim_block_ids_to_incremerkle_in_state)
        .and_then(get_active_schedule_from_db_and_add_to_state)
        .and_then(validate_producer_slot_of_block_in_state)
        .and_then(validate_block_header_signature)
        .and_then(maybe_add_new_eos_schedule_to_db_and_return_state)
        .and_then(get_processed_global_sequences_and_add_to_state)
        .and_then(maybe_filter_duplicate_proofs_from_state)
        .and_then(maybe_filter_out_proofs_for_wrong_eos_account_name)
        .and_then(maybe_filter_out_action_proof_receipt_mismatches_and_return_state)
        .and_then(maybe_filter_out_invalid_action_receipt_digests)
        .and_then(maybe_filter_out_proofs_with_invalid_merkle_proofs)
        .and_then(maybe_filter_out_proofs_with_wrong_action_mroot)
        .and_then(maybe_filter_proofs_for_v1_redeem_actions)
        .and_then(maybe_parse_btc_tx_infos_and_put_in_state)
        .and_then(maybe_filter_value_too_low_btc_tx_infos_in_state)
        .and_then(maybe_filter_out_already_processed_tx_ids_from_state)
        .and_then(maybe_add_global_sequences_to_processed_list_and_return_state)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_sign_txs_and_add_to_state)
        .and_then(maybe_filter_btc_txs_in_state)
        .and_then(maybe_increment_btc_signature_nonce_and_return_eos_state)
        .and_then(maybe_extract_btc_utxo_from_btc_tx_in_state)
        .and_then(maybe_save_btc_utxos_to_db)
        .and_then(save_latest_block_id_to_db)
        .and_then(save_latest_block_num_to_db)
        .and_then(save_incremerkle_from_state_to_db)
        .and_then(end_eos_db_transaction_and_return_state)
        .and_then(get_eos_output)
}
