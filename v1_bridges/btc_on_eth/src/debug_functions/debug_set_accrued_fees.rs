use common::{fees::fee_database_utils::FeeDatabaseUtils, traits::DatabaseInterface, types::Result};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
use serde_json::json;

use crate::constants::CORE_TYPE;

/// # Debug Set Accrued Fees
///
/// Allows manual setting of the accured fees stored in the database for this core.
#[named]
pub fn debug_set_accrued_fees<D: DatabaseInterface>(db: &D, amount: u64, signature: &str) -> Result<String> {
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &amount)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| {
            let fee_db_utils = FeeDatabaseUtils::new_for_btc_on_eth();
            fee_db_utils.reset_accrued_fees(db)?;
            fee_db_utils.increment_accrued_fees(db, amount)?;
            Ok(())
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true,"accrued_fee":amount}).to_string())
}
