use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    check_core_is_initialized::check_core_is_initialized_and_return_eos_state,
    eos::{
        eos_state::EosState,
        get_eos_output::get_eos_output,
        save_btc_utxos_to_db::maybe_save_btc_utxos_to_db,
        sign_transactions::maybe_sign_txs_and_add_to_state,
        increment_signature_nonce::maybe_increment_signature_nonce,
        get_processed_tx_ids::get_processed_tx_ids_and_add_to_state,
        parse_redeem_params::maybe_parse_redeem_params_and_put_in_state,
        filter_duplicate_proofs::maybe_filter_duplicate_proofs_from_state,
        add_tx_ids_to_processed_list::maybe_add_tx_ids_to_processed_tx_ids,
        parse_submission_material::parse_submission_material_and_add_to_state,
        filter_invalid_action_digests::maybe_filter_out_invalid_action_digests,
        filter_irrelevant_proofs::maybe_filter_out_irrelevant_proofs_from_state,
        extract_utxos_from_btc_txs::maybe_extract_btc_utxo_from_btc_tx_in_state,
        filter_merkle_proofs_with_wrong_root::{
            maybe_filter_out_proofs_with_wrong_merkle_roots,
        },
        filter_redeem_params::{
            maybe_filter_value_too_low_redeem_params_in_state,
        },
        filter_action_and_receipt_mismatches::{
            maybe_filter_out_action_proof_receipt_mismatches,
        },
        filter_already_processed_txs::{
            maybe_filter_out_already_processed_tx_ids_from_state,
        },
        eos_database_utils::{
            end_eos_db_transaction,
            start_eos_db_transaction,
        },
    },
};

pub fn submit_eos_block_to_core<D>(
    db: D,
    block_json: String,
) -> Result<String>
    where D: DatabaseInterface
{
    parse_submission_material_and_add_to_state(block_json, EosState::init(db))
        .and_then(check_core_is_initialized_and_return_eos_state)
        .and_then(get_processed_tx_ids_and_add_to_state)
        .and_then(start_eos_db_transaction)
        .and_then(maybe_filter_duplicate_proofs_from_state)
        .and_then(maybe_filter_out_irrelevant_proofs_from_state)
        .and_then(maybe_filter_out_action_proof_receipt_mismatches)
        .and_then(maybe_filter_out_invalid_action_digests)
        .and_then(maybe_filter_out_proofs_with_wrong_merkle_roots)
        //.and_then(validate_block_header_signatures)
        .and_then(maybe_parse_redeem_params_and_put_in_state)
        .and_then(maybe_filter_value_too_low_redeem_params_in_state)
        .and_then(maybe_filter_out_already_processed_tx_ids_from_state)
        .and_then(maybe_add_tx_ids_to_processed_tx_ids)
        .and_then(maybe_sign_txs_and_add_to_state)
        .and_then(maybe_increment_signature_nonce)
        .and_then(maybe_extract_btc_utxo_from_btc_tx_in_state)
        .and_then(maybe_save_btc_utxos_to_db)
        .and_then(end_eos_db_transaction)
        .and_then(get_eos_output)
}
