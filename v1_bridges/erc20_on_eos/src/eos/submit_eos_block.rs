use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eos::{
    append_interim_block_ids_to_incremerkle_in_state,
    end_eos_db_transaction_and_return_state,
    get_active_schedule_from_db_and_add_to_state,
    get_enabled_protocol_features_and_add_to_state,
    get_incremerkle_and_add_to_state,
    get_processed_global_sequences_and_add_to_state,
    maybe_add_global_sequences_to_processed_list_and_return_state,
    maybe_add_new_eos_schedule_to_db_and_return_state,
    maybe_filter_duplicate_proofs_from_state,
    maybe_filter_out_action_proof_receipt_mismatches_and_return_state,
    maybe_filter_out_invalid_action_receipt_digests,
    maybe_filter_out_proofs_for_accounts_not_in_token_dictionary,
    maybe_filter_out_proofs_with_invalid_merkle_proofs,
    maybe_filter_out_proofs_with_wrong_action_mroot,
    maybe_filter_proofs_for_v1_redeem_actions,
    parse_submission_material_and_add_to_state,
    save_incremerkle_from_state_to_db,
    save_latest_block_id_to_db,
    save_latest_block_num_to_db,
    validate_block_header_signature,
    validate_producer_slot_of_block_in_state,
    EosState,
};

use crate::eos::{
    account_for_fees::maybe_account_for_fees,
    divert_to_safe_address::{
        maybe_divert_txs_to_safe_address_if_destination_is_token_address,
        maybe_divert_txs_to_safe_address_if_destination_is_vault_address,
    },
    eth_tx_info::{maybe_filter_out_already_processed_tx_ids_from_state, maybe_parse_eth_tx_infos_and_put_in_state},
    get_eos_output::get_eos_output,
    increment_eth_nonce::maybe_increment_eth_nonce_in_db_and_return_eos_state,
    sign_normal_eth_txs::maybe_sign_normal_eth_txs_and_add_to_state,
};

/// # Submit EOS Block to Core
///
/// The main submission pipeline. Submitting an EOS block to the enclave will - if the block is
/// valid & the accompanying transaction IDs update the incremerkle correctly - advanced the core's
/// incremerkle accordingly. Any proofs submitted with the block and transaction IDs will then be
/// parsed and if found to pertain to peg outs made in the block in question, an ETH transaction
/// will be signed and returned to the caller.
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
        .and_then(|state| state.get_eos_eth_token_dictionary_and_add_to_state())
        .and_then(maybe_add_new_eos_schedule_to_db_and_return_state)
        .and_then(get_processed_global_sequences_and_add_to_state)
        .and_then(maybe_filter_duplicate_proofs_from_state)
        .and_then(maybe_filter_out_proofs_for_accounts_not_in_token_dictionary)
        .and_then(maybe_filter_out_action_proof_receipt_mismatches_and_return_state)
        .and_then(maybe_filter_out_invalid_action_receipt_digests)
        .and_then(maybe_filter_out_proofs_with_invalid_merkle_proofs)
        .and_then(maybe_filter_out_proofs_with_wrong_action_mroot)
        .and_then(maybe_filter_proofs_for_v1_redeem_actions)
        .and_then(maybe_parse_eth_tx_infos_and_put_in_state)
        .and_then(maybe_filter_out_already_processed_tx_ids_from_state)
        .and_then(maybe_add_global_sequences_to_processed_list_and_return_state)
        .and_then(maybe_account_for_fees)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_vault_address)
        .and_then(maybe_sign_normal_eth_txs_and_add_to_state)
        .and_then(maybe_increment_eth_nonce_in_db_and_return_eos_state)
        .and_then(save_latest_block_id_to_db)
        .and_then(save_latest_block_num_to_db)
        .and_then(save_incremerkle_from_state_to_db)
        .and_then(end_eos_db_transaction_and_return_state)
        .and_then(get_eos_output)
}
