use serde_json::json;

use crate::{
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    fees::fee_database_utils::FeeDatabaseUtils,
    traits::DatabaseInterface,
    types::Result,
};

/// # Debug Set Accrued Fees
///
/// Allows manual setting of the accured fees stored in the database for this core.
pub fn debug_set_accrued_fees<D: DatabaseInterface>(
    db: &D,
    amount: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnEth, signature, debug_command_hash))
        .and_then(|_| {
            let fee_db_utils = FeeDatabaseUtils::new_for_btc_on_eth();
            fee_db_utils.reset_accrued_fees(db)?;
            fee_db_utils.increment_accrued_fees(db, amount)?;
            Ok(())
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true,"accrued_fee":amount}).to_string())
}
