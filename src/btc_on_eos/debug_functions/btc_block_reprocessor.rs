pub use serde_json::json;

use crate::{
    btc_on_eos::{
        btc::{
            account_for_fees::maybe_account_for_fees as maybe_account_for_peg_in_fees,
            divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_token_address,
            eos_tx_info::parse_eos_tx_infos_from_p2sh_deposits_and_add_to_state,
            get_btc_output_json::{get_btc_output_as_string, get_eos_signed_tx_info, BtcOutput},
            sign_transactions::get_signed_eos_ptoken_issue_txs,
        },
        check_core_is_initialized::check_core_is_initialized_and_return_btc_state,
    },
    chains::{
        btc::{
            btc_database_utils::{end_btc_db_transaction, start_btc_db_transaction},
            btc_state::BtcState,
            btc_submission_material::parse_submission_material_and_put_in_state,
            filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
            get_btc_block_in_db_format::create_btc_block_in_db_format_and_put_in_state,
            get_deposit_info_hash_map::get_deposit_info_hash_map_and_put_in_state,
            increment_eos_nonce::maybe_increment_eos_nonce,
            validate_btc_block_header::validate_btc_block_header_in_state,
            validate_btc_difficulty::validate_difficulty_of_btc_block_in_state,
            validate_btc_merkle_root::validate_btc_merkle_root,
            validate_btc_proof_of_work::validate_proof_of_work_of_btc_block_in_state,
        },
        eos::eos_crypto::eos_private_key::EosPrivateKey,
    },
    debug_mode::check_debug_mode,
    fees::fee_database_utils::FeeDatabaseUtils,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

fn debug_reprocess_btc_block_for_stale_eos_tx_maybe_accruing_fees<D: DatabaseInterface>(
    db: D,
    block_json_string: &str,
    accrue_fees: bool,
) -> Result<String> {
    info!(
        "✔ Reprocessing BTC block to core {} fees accruing",
        if accrue_fees { "WITH" } else { "WITHOUT" }
    );
    check_debug_mode()
        .and_then(|_| parse_submission_material_and_put_in_state(block_json_string, BtcState::init(&db)))
        .and_then(check_core_is_initialized_and_return_btc_state)
        .and_then(start_btc_db_transaction)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_difficulty_of_btc_block_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(parse_eos_tx_infos_from_p2sh_deposits_and_add_to_state)
        .and_then(create_btc_block_in_db_format_and_put_in_state)
        .and_then(|state| {
            if accrue_fees {
                maybe_account_for_peg_in_fees(state)
            } else {
                info!("✔ Accounting for fees in signing params but NOT accruing them!");
                let basis_points = FeeDatabaseUtils::new_for_btc_on_eos().get_peg_in_basis_points_from_db(state.db)?;
                let updated_tx_infos = state.btc_on_eos_eos_tx_infos.subtract_fees(basis_points)?;
                state.replace_btc_on_eos_eos_tx_infos(updated_tx_infos)
            }
        })
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(|state| {
            info!("✔ Maybe signing reprocessed `BtcOnEosEosTxInfos`...");
            let eos_signed_txs = get_signed_eos_ptoken_issue_txs(
                state.get_eos_ref_block_num()?,
                state.get_eos_ref_block_prefix()?,
                &state.eos_db_utils.get_eos_chain_id_from_db()?,
                &EosPrivateKey::get_from_db(state.db)?,
                &state.eos_db_utils.get_eos_account_name_string_from_db()?,
                &state.btc_on_eos_eos_tx_infos,
                &state.btc_db_utils.get_btc_chain_id_from_db()?,
            )?;
            info!("✔ EOS signed txs: {:?}", eos_signed_txs);
            state.add_eos_signed_txs(eos_signed_txs)
        })
        .and_then(maybe_increment_eos_nonce)
        .and_then(|state| {
            info!("✔ Getting BTC output json and putting in state...");
            let output = serde_json::to_string(&BtcOutput {
                btc_latest_block_number: state.btc_db_utils.get_btc_latest_block_from_db()?.height,
                eos_signed_transactions: match &state.eos_signed_txs.len() {
                    0 => vec![],
                    _ => get_eos_signed_tx_info(
                        &state.eos_signed_txs,
                        &state.btc_on_eos_eos_tx_infos,
                        state.eos_db_utils.get_eos_account_nonce_from_db()?,
                    )?,
                },
            })?;
            state.add_output_json_string(output)
        })
        .and_then(end_btc_db_transaction)
        .and_then(get_btc_output_as_string)
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Reprocess BTC Block For Stale Transaction
///
/// This function takes BTC block submission material and runs it thorugh the BTC submission
/// pipeline signing any transactions along the way. The `stale_transaction` part alludes to the
/// fact that EOS transactions have an intrinsic time limit, meaning a failure of upstream parts of
/// the bridge (ie tx broadcasting) could lead to expired transactions that can't ever be mined.
///
/// ### NOTE:
///
/// This version of the function _will_ account for fees so the outputted transaction's value is
/// correct, but it will __NOT__ accrue those fees onto the balance stored in the encrypted database.
/// This is to not double-count the fee if this block had already had a failed processing via an
/// organic block submission.
///
/// ### BEWARE:
/// This function WILL increment the EOS nonce if transactions are signed. The user of this function
/// should understand what this means when inserting the report outputted from this debug function.
/// If this output is to _replace_ an existing report, the nonces in the report and in the core's
/// database should be modified accordingly.
pub fn debug_reprocess_btc_block_for_stale_eos_tx<D: DatabaseInterface>(
    db: D,
    block_json_string: &str,
) -> Result<String> {
    debug_reprocess_btc_block_for_stale_eos_tx_maybe_accruing_fees(db, block_json_string, false)
}

/// # Debug Reprocess BTC Block For Stale Transaction
///
/// This function takes BTC block submission material and runs it thorugh the BTC submission
/// pipeline signing any transactions along the way. The `stale_transaction` part alludes to the
/// fact that EOS transactions have an intrinsic time limit, meaning a failure of upstream parts of
/// the bridge (ie tx broadcasting) could lead to expired transactions that can't ever be mined.
///
/// ### NOTE:
///
/// This version of the function _will_ account for fees so the outputted transaction's value is
/// correct, and will also add those fees to the `accrued_fees` value stored in the encrypted
/// database. Only use this function if you're sure those fees have not already been accrued from
/// the blocks organic submission to the core.
///
/// ### BEWARE:
/// This function WILL increment the EOS nonce if transactions are signed. The user of this function
/// should understand what this means when inserting the report outputted from this debug function.
/// If this output is to _replace_ an existing report, the nonces in the report and in the core's
/// database should be modified accordingly.
pub fn debug_reprocess_btc_block_for_stale_eos_tx_with_fee_accrual<D: DatabaseInterface>(
    db: D,
    block_json_string: &str,
) -> Result<String> {
    debug_reprocess_btc_block_for_stale_eos_tx_maybe_accruing_fees(db, block_json_string, true)
}
