use common::{traits::DatabaseInterface, types::Result, utils::prepend_debug_output_marker_to_string};
use common_btc::{get_btc_on_eth_fee_withdrawal_tx, get_hex_tx_from_signed_btc_tx, BtcDbUtils};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
use serde_json::json;

use crate::{constants::CORE_TYPE, eth::extract_change_utxo_from_btc_tx_and_save_in_db};

/// # Debug Get Fee Withdrawal Tx
///
/// This function crates a BTC transaction to the passed in address for the amount of accrued fees
/// accounted for in the encrypted database. The function then reset this value back to zero. The
/// signed transaction is returned to the caller.
#[named]
pub fn debug_get_fee_withdrawal_tx<D: DatabaseInterface>(db: &D, btc_address: &str, signature: &str) -> Result<String> {
    info!("✔ Debug getting `btc-on-eth` withdrawal tx...");
    let btc_db_utils = BtcDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), btc_address)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
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
