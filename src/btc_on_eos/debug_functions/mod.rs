pub use serde_json::json;

pub(crate) mod btc_block_reprocessor;
pub(crate) mod eos_block_reprocessor;

use crate::{
    btc_on_eos::check_core_is_initialized::check_core_is_initialized_and_return_btc_state,
    chains::btc::{
        btc_database_utils::{end_btc_db_transaction, BtcDbUtils},
        btc_state::BtcState,
        btc_submission_material::parse_submission_material_and_put_in_state,
        btc_utils::{get_hex_tx_from_signed_btc_tx, get_pay_to_pub_key_hash_script},
        extract_utxos_from_p2pkh_txs::{
            extract_utxos_from_p2pkh_txs,
            maybe_extract_utxos_from_p2pkh_txs_and_put_in_btc_state,
        },
        extract_utxos_from_p2sh_txs::maybe_extract_utxos_from_p2sh_txs_and_put_in_state,
        filter_p2pkh_deposit_txs::filter_for_p2pkh_deposit_txs_including_change_outputs_and_add_to_state,
        filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
        filter_utxos::filter_out_utxos_extant_in_db_from_state,
        get_deposit_info_hash_map::get_deposit_info_hash_map_and_put_in_state,
        save_utxos_to_db::maybe_save_utxos_to_db,
        utxo_manager::utxo_database_utils::save_utxos_to_db,
        validate_btc_block_header::validate_btc_block_header_in_state,
        validate_btc_difficulty::validate_difficulty_of_btc_block_in_state,
        validate_btc_merkle_root::validate_btc_merkle_root,
        validate_btc_proof_of_work::validate_proof_of_work_of_btc_block_in_state,
    },
    constants::SUCCESS_JSON,
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    fees::{
        fee_database_utils::FeeDatabaseUtils,
        fee_utils::sanity_check_basis_points_value,
        fee_withdrawals::get_btc_on_eos_fee_withdrawal_tx,
    },
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Maybe Add UTXO To DB
///
/// This function accepts as its param BTC submission material, in which it inspects all the
/// transactions looking for any pertaining to the core's own public key, or deposit addresses
/// derived from it. Any it finds it will extract the UTXO from and add it to the encrypted
/// database. Note that this fxn WILL extract the enclave's own change UTXOs from blocks!
///
/// ### NOTE:
/// The core won't accept UTXOs it already has in its encrypted database.
pub fn debug_maybe_add_utxo_to_db<D: DatabaseInterface>(
    db: &D,
    btc_submission_material_json: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnEos, signature, debug_command_hash))
        .and_then(|_| parse_submission_material_and_put_in_state(btc_submission_material_json, BtcState::init(db)))
        .and_then(check_core_is_initialized_and_return_btc_state)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_difficulty_of_btc_block_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(filter_for_p2pkh_deposit_txs_including_change_outputs_and_add_to_state)
        .and_then(maybe_extract_utxos_from_p2pkh_txs_and_put_in_btc_state)
        .and_then(maybe_extract_utxos_from_p2sh_txs_and_put_in_state)
        .and_then(filter_out_utxos_extant_in_db_from_state)
        .and_then(maybe_save_utxos_to_db)
        .and_then(end_btc_db_transaction)
        .map(|_| SUCCESS_JSON.to_string())
        .map(prepend_debug_output_marker_to_string)
}

fn debug_put_btc_on_eos_basis_points_in_db<D: DatabaseInterface>(
    db: &D,
    basis_points: u64,
    peg_in: bool,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let suffix = if peg_in { "in" } else { "out" };
    info!(
        "✔ Debug setting `BtcOnEos` peg-{} basis-points to {}",
        suffix, basis_points
    );
    check_debug_mode()
        .and_then(|_| sanity_check_basis_points_value(basis_points))
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnEos, signature, debug_command_hash))
        .and_then(|_| {
            if peg_in {
                FeeDatabaseUtils::new_for_btc_on_eos().put_peg_in_basis_points_in_db(db, basis_points)
            } else {
                FeeDatabaseUtils::new_for_btc_on_eos().put_peg_out_basis_points_in_db(db, basis_points)
            }
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(
            json!({format!("set_btc_on_eos_peg_{}_basis_points", suffix):true}).to_string()
        ))
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Put BTC-on-EOS Peg-In Basis-Points In DB
///
/// This function sets to the given value the `BTC-on-EOS` peg-in basis-points in the encrypted
/// database.
pub fn debug_put_btc_on_eos_peg_in_basis_points_in_db<D: DatabaseInterface>(
    db: &D,
    basis_points: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug setting `BtcOnEos` peg-in basis-points to {}", basis_points);
    debug_put_btc_on_eos_basis_points_in_db(db, basis_points, true, signature, debug_command_hash)
}

/// # Debug Put BTC-on-EOS Peg-Out Basis-Points In DB
///
/// This function sets to the given value the `BTC-on-EOS` peg-out basis-points in the encrypted
/// database.
pub fn debug_put_btc_on_eos_peg_out_basis_points_in_db<D: DatabaseInterface>(
    db: &D,
    basis_points: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug setting `BtcOnEos` peg-out basis-points to {}", basis_points);
    debug_put_btc_on_eos_basis_points_in_db(db, basis_points, false, signature, debug_command_hash)
}

/// # Debug Get Fee Withdrawal Tx
///
/// This function crates a BTC transaction to the passed in address for the amount of accrued fees
/// accounted for in the encrypted database. The function then reset this value back to zero. The
/// signed transaction is returned to the caller.
pub fn debug_get_fee_withdrawal_tx<D: DatabaseInterface>(
    db: &D,
    btc_address: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug getting `BtcOnEos` withdrawal tx...");
    let btc_db_utils = BtcDbUtils::new(db);
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnEos, signature, debug_command_hash))
        .and_then(|_| get_btc_on_eos_fee_withdrawal_tx(db, btc_address))
        .and_then(|btc_tx| {
            let change_utxos = get_pay_to_pub_key_hash_script(&btc_db_utils.get_btc_address_from_db()?)
                .map(|target_script| extract_utxos_from_p2pkh_txs(&target_script, &[btc_tx.clone()]))?;
            save_utxos_to_db(db, &change_utxos)?;
            db.end_transaction()?;
            Ok(json!({ "signed_btc_tx": get_hex_tx_from_signed_btc_tx(&btc_tx) }).to_string())
        })
        .map(prepend_debug_output_marker_to_string)
}
