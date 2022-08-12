use serde_json::json;

use crate::{
    btc_on_eth::eth::extract_change_utxo_from_btc_tx_and_save_in_db,
    chains::btc::{btc_database_utils::BtcDbUtils, btc_utils::get_hex_tx_from_signed_btc_tx},
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    fees::fee_withdrawals::get_btc_on_eth_fee_withdrawal_tx,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

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
    info!("✔ Debug getting `btc-on-eth` withdrawal tx...");
    let btc_db_utils = BtcDbUtils::new(db);
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnEth, signature, debug_command_hash))
        .and_then(|_| get_btc_on_eth_fee_withdrawal_tx(db, btc_address))
        .and_then(|btc_tx| {
            extract_change_utxo_from_btc_tx_and_save_in_db(
                db,
                &btc_db_utils.get_btc_address_from_db()?,
                btc_tx.clone(),
            )?;
            db.end_transaction()?;
            Ok(json!({ "signed_btc_tx": get_hex_tx_from_signed_btc_tx(&btc_tx) }).to_string())
        })
        .map(prepend_debug_output_marker_to_string)
}
