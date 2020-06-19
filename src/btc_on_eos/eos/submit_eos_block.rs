use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::{
        check_core_is_initialized::{
            check_core_is_initialized_and_return_eos_state,
        },
        eos::{
            eos_state::EosState,
            get_eos_output::get_eos_output,
            add_schedule::maybe_add_new_eos_schedule_to_db,
            save_btc_utxos_to_db::maybe_save_btc_utxos_to_db,
            save_latest_block_id::save_latest_block_id_to_db,
            save_latest_block_num::save_latest_block_num_to_db,
            sign_transactions::maybe_sign_txs_and_add_to_state,
            save_incremerkle::save_incremerkle_from_state_to_db,
            validate_signature::validate_block_header_signature,
            get_eos_incremerkle::get_incremerkle_and_add_to_state,
            increment_signature_nonce::maybe_increment_signature_nonce,
            get_processed_tx_ids::get_processed_tx_ids_and_add_to_state,
            parse_redeem_params::maybe_parse_redeem_params_and_put_in_state,
            validate_producer_slot::validate_producer_slot_of_block_in_state,
            get_active_schedule::get_active_schedule_from_db_and_add_to_state,
            filter_duplicate_proofs::maybe_filter_duplicate_proofs_from_state,
            parse_submission_material::parse_submission_material_and_add_to_state,
            extract_utxos_from_btc_txs::maybe_extract_btc_utxo_from_btc_tx_in_state,
            filter_redeem_params::maybe_filter_value_too_low_redeem_params_in_state,
            filter_irrelevant_proofs::maybe_filter_out_irrelevant_proofs_from_state,
            append_interim_block_ids::append_interim_block_ids_to_incremerkle_in_state,
            get_enabled_protocol_features::get_enabled_protocol_features_and_add_to_state,
            filter_invalid_action_digests::maybe_filter_out_invalid_action_receipt_digests,
            filter_invalid_merkle_proofs::maybe_filter_out_proofs_with_invalid_merkle_proofs,
            filter_already_processed_txs::maybe_filter_out_already_processed_tx_ids_from_state,
            add_global_sequences_to_processed_list::maybe_add_global_sequences_to_processed_list,
            filter_proofs_with_wrong_action_mroot::maybe_filter_out_proofs_with_wrong_action_mroot,
            filter_action_and_receipt_mismatches::maybe_filter_out_action_proof_receipt_mismatches,
            eos_database_utils::{
                end_eos_db_transaction,
                start_eos_db_transaction,
            },
        },
    },
};

pub fn submit_eos_block_to_core<D>(
    db: D,
    block_json: &str,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Submitting EOS block to core...");
    parse_submission_material_and_add_to_state(block_json, EosState::init(db))
        .and_then(check_core_is_initialized_and_return_eos_state)
        .and_then(get_enabled_protocol_features_and_add_to_state)
        .and_then(get_incremerkle_and_add_to_state)
        .and_then(append_interim_block_ids_to_incremerkle_in_state)
        .and_then(get_active_schedule_from_db_and_add_to_state)
        .and_then(validate_producer_slot_of_block_in_state)
        .and_then(validate_block_header_signature)
        .and_then(start_eos_db_transaction)
        .and_then(maybe_add_new_eos_schedule_to_db)
        .and_then(get_processed_tx_ids_and_add_to_state)
        .and_then(maybe_filter_duplicate_proofs_from_state)
        .and_then(maybe_filter_out_irrelevant_proofs_from_state)
        .and_then(maybe_filter_out_action_proof_receipt_mismatches)
        .and_then(maybe_filter_out_invalid_action_receipt_digests)
        .and_then(maybe_filter_out_proofs_with_invalid_merkle_proofs)
        .and_then(maybe_filter_out_proofs_with_wrong_action_mroot)
        .and_then(maybe_parse_redeem_params_and_put_in_state)
        .and_then(maybe_filter_value_too_low_redeem_params_in_state)
        .and_then(maybe_filter_out_already_processed_tx_ids_from_state)
        .and_then(maybe_add_global_sequences_to_processed_list)
        .and_then(maybe_sign_txs_and_add_to_state)
        .and_then(maybe_increment_signature_nonce)
        .and_then(maybe_extract_btc_utxo_from_btc_tx_in_state)
        .and_then(maybe_save_btc_utxos_to_db)
        .and_then(save_latest_block_id_to_db)
        .and_then(save_latest_block_num_to_db)
        .and_then(save_incremerkle_from_state_to_db)
        .and_then(end_eos_db_transaction)
        .and_then(get_eos_output)
}
