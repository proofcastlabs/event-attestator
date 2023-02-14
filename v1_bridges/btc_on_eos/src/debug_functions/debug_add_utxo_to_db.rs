use common::{
    constants::SUCCESS_JSON,
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_btc::{
    end_btc_db_transaction,
    filter_for_p2pkh_deposit_txs_including_change_outputs_and_add_to_state,
    filter_out_utxos_extant_in_db_from_state,
    filter_p2sh_deposit_txs_and_add_to_state,
    get_deposit_info_hash_map_and_put_in_state,
    maybe_extract_utxos_from_p2pkh_txs_and_put_in_btc_state,
    maybe_extract_utxos_from_p2sh_txs_and_put_in_state,
    maybe_save_utxos_to_db,
    parse_submission_material_and_put_in_state,
    validate_btc_block_header_in_state,
    validate_btc_merkle_root,
    validate_difficulty_of_btc_block_in_state,
    validate_proof_of_work_of_btc_block_in_state,
    BtcState,
};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;

use crate::constants::CORE_TYPE;

// NOTE: Some functions in here have their debug signature requirement temporarily removed, to
// allow for the automated `ptokens-utxo-recovery` tool to work. Once that tool has been updated to
// provide correct signatures, the signatures required for these functions will be re-instated.

const SKIP_DEBUG_SIGNATURE_CHECK: bool = true;

/// # Debug Maybe Add UTXO To DB
///
/// This function accepts as its param BTC submission material, in which it inspects all the
/// transactions looking for any pertaining to the core's own public key, or deposit addresses
/// derived from it. Any it finds it will extract the UTXO from and add it to the encrypted
/// database. Note that this fxn WILL extract the enclave's own change UTXOs from blocks!
///
/// ### NOTE:
/// The core won't accept UTXOs it already has in its encrypted database.
#[named]
pub fn debug_maybe_add_utxo_to_db<D: DatabaseInterface>(
    db: &D,
    btc_submission_material_json: &str,
    signature: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), &btc_submission_material_json)())
        .and_then(|hash| {
            if SKIP_DEBUG_SIGNATURE_CHECK {
                warn!("✘ Debug signature check SKIPPED for fxn: {}", function_name!());
                Ok(())
            } else {
                validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test))
            }
        })
        .and_then(|_| parse_submission_material_and_put_in_state(btc_submission_material_json, BtcState::init(db)))
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
