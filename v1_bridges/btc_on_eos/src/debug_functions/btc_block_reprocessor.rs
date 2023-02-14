use common::{
    core_type::CoreType,
    traits::{DatabaseInterface, Serdable},
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_btc::{
    create_btc_block_in_db_format_and_put_in_state,
    end_btc_db_transaction,
    filter_p2sh_deposit_txs_and_add_to_state,
    get_deposit_info_hash_map_and_put_in_state,
    parse_submission_material_and_put_in_state,
    validate_btc_block_header_in_state,
    validate_btc_merkle_root,
    validate_difficulty_of_btc_block_in_state,
    validate_proof_of_work_of_btc_block_in_state,
    BtcState,
};
use common_debug_signers::validate_debug_command_signature;
use common_eos::{EosDbUtils, EosPrivateKey, EosSignedTransactions};
use common_fees::FeeDatabaseUtils;
use function_name::named;
pub use serde_json::json;

use crate::{
    btc::{
        get_btc_output_as_string,
        get_eos_signed_tx_info,
        get_signed_eos_ptoken_issue_txs,
        maybe_account_for_peg_in_fees,
        maybe_divert_txs_to_safe_address_if_destination_is_token_address,
        maybe_increment_eos_nonce,
        parse_eos_tx_infos_from_p2sh_deposits_and_add_to_state,
        BtcOnEosEosTxInfos,
        BtcOutput,
    },
    constants::CORE_TYPE,
};

#[named]
fn debug_reprocess_btc_block_for_stale_eos_tx_maybe_accruing_fees<D: DatabaseInterface>(
    db: &D,
    block_json_str: &str,
    accrue_fees: bool,
    signature: &str,
) -> Result<String> {
    info!(
        "✔ Reprocessing BTC block to core {} fees accruing",
        if accrue_fees { "WITH" } else { "WITHOUT" }
    );
    let eos_db_utils = EosDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json_str, &accrue_fees)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| parse_submission_material_and_put_in_state(block_json_str, BtcState::init(db)))
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
                let updated_tx_infos = BtcOnEosEosTxInfos::from_bytes(&state.tx_infos)?
                    .subtract_fees(basis_points)?
                    .to_bytes()?;
                Ok(state.add_tx_infos(updated_tx_infos))
            }
        })
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(|state| {
            info!("✔ Maybe signing reprocessed `BtcOnEosEosTxInfos`...");
            let eos_signed_txs = get_signed_eos_ptoken_issue_txs(
                state.get_eos_ref_block_num()?,
                state.get_eos_ref_block_prefix()?,
                &eos_db_utils.get_eos_chain_id_from_db()?,
                &EosPrivateKey::get_from_db(state.db)?,
                &eos_db_utils.get_eos_account_name_string_from_db()?,
                &BtcOnEosEosTxInfos::from_bytes(&state.tx_infos)?,
                &state.btc_db_utils.get_btc_chain_id_from_db()?,
            )?;
            info!("✔ EOS signed txs: {:?}", eos_signed_txs);
            Ok(state.add_eos_signed_txs(eos_signed_txs.to_bytes()?))
        })
        .and_then(maybe_increment_eos_nonce)
        .and_then(|state| {
            info!("✔ Getting BTC output json and putting in state...");
            let output = serde_json::to_string(&BtcOutput {
                btc_latest_block_number: state.btc_db_utils.get_btc_latest_block_from_db()?.height,
                eos_signed_transactions: if state.eos_signed_txs.is_empty() {
                    vec![]
                } else {
                    get_eos_signed_tx_info(
                        &EosSignedTransactions::from_bytes(&state.eos_signed_txs)?,
                        &BtcOnEosEosTxInfos::from_bytes(&state.tx_infos)?,
                        eos_db_utils.get_eos_account_nonce_from_db()?,
                    )?
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
    db: &D,
    block_json_str: &str,
    signature: &str,
) -> Result<String> {
    debug_reprocess_btc_block_for_stale_eos_tx_maybe_accruing_fees(db, block_json_str, false, signature)
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
    db: &D,
    block_json_str: &str,
    signature: &str,
) -> Result<String> {
    debug_reprocess_btc_block_for_stale_eos_tx_maybe_accruing_fees(db, block_json_str, true, signature)
}
