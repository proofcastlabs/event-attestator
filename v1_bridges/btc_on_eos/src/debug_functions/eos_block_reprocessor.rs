use common::{
    chains::eos::{
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
        get_enabled_protocol_features::get_enabled_protocol_features_and_add_to_state,
        EosState,
    },
    fees::fee_database_utils::FeeDatabaseUtils,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
    CoreType,
};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
pub use serde_json::json;

use crate::{
    constants::CORE_TYPE,
    eos::{
        get_eos_output,
        maybe_account_for_peg_out_fees,
        maybe_extract_btc_utxo_from_btc_tx_in_state,
        maybe_filter_btc_txs_in_state,
        maybe_filter_value_too_low_btc_tx_infos_in_state,
        maybe_increment_btc_signature_nonce_and_return_eos_state,
        maybe_parse_btc_tx_infos_and_put_in_state,
        maybe_save_btc_utxos_to_db,
        maybe_sign_txs_and_add_to_state,
        BtcOnEosBtcTxInfos,
    },
};

#[named]
fn debug_reprocess_eos_block_maybe_accruing_fees<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    accrue_fees: bool,
    signature: &str,
) -> Result<String> {
    info!(
        "✔ Debug reprocessing EOS block {} fees accruing!",
        if accrue_fees { "WITH" } else { "WITHOUT" }
    );
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json, &accrue_fees)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| parse_submission_material_and_add_to_state(block_json, EosState::init(db)))
        .and_then(get_enabled_protocol_features_and_add_to_state)
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
        .and_then(maybe_add_global_sequences_to_processed_list_and_return_state)
        .and_then(|state| {
            if accrue_fees {
                maybe_account_for_peg_out_fees(state)
            } else {
                info!("✔ Accounting for fees in signing params but NOT accruing them!");
                let basis_points = FeeDatabaseUtils::new_for_btc_on_eos().get_peg_out_basis_points_from_db(state.db)?;
                if state.tx_infos.is_empty() {
                    Ok(state)
                } else {
                    BtcOnEosBtcTxInfos::from_bytes(&state.tx_infos)
                        .and_then(|infos| infos.subtract_fees(basis_points))
                        .and_then(|infos| infos.to_bytes())
                        .map(|bytes| state.add_tx_infos(bytes))
                }
            }
        })
        .and_then(maybe_sign_txs_and_add_to_state)
        .and_then(maybe_filter_btc_txs_in_state)
        .and_then(maybe_increment_btc_signature_nonce_and_return_eos_state)
        .and_then(maybe_extract_btc_utxo_from_btc_tx_in_state)
        .and_then(maybe_save_btc_utxos_to_db)
        .and_then(end_eos_db_transaction_and_return_state)
        .and_then(get_eos_output)
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Reprocess EOS Block For Stale Transaction
///
/// This function will take a passed in EOS block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block
///
/// ### NOTE:
///
/// This version of the function _will_ account for fees so the outputted transaction's value is
/// correct, and will also add those fees to the `accrued_fees` value stored in the encrypted
/// database. Only use this function if you're sure those fees have not already been accrued from
/// the blocks organic submission to the core.
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, ALL future BTC transactions will
/// fail due to the core having an incorret set of UTXOs!
pub fn debug_reprocess_eos_block_with_fee_accrual<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    signature: &str,
) -> Result<String> {
    debug_reprocess_eos_block_maybe_accruing_fees(db, block_json, true, signature)
}

/// # Debug Reprocess EOS Block For Stale Transaction
///
/// This function will take a passed in EOS block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block
///
/// ### NOTE:
///
/// This version of the function _will_ account for fees so the outputted transaction's value is
/// correct, but it will __NOT__ accrue those fees onto the balance stored in the encrypted database.
/// This is to not double-count the fee if this block had already had a failed processing via an
/// organic block submission.
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, ALL future BTC transactions will
/// fail due to the core having an incorret set of UTXOs!
pub fn debug_reprocess_eos_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    debug_reprocess_eos_block_maybe_accruing_fees(db, block_json, false, signature)
}
