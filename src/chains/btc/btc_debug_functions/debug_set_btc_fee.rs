use function_name::named;
use serde_json::json;

use crate::{
    chains::btc::btc_database_utils::BtcDbUtils,
    core_type::CoreType,
    debug_functions::validate_debug_command_signature,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Put BTC Fee In Db
///
/// This function sets the BTC fee in the encrypted database to the given value. The unit is
/// satoshis-per-byte.
#[named]
pub fn debug_set_btc_fee<D: DatabaseInterface>(
    db: &D,
    fee: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug putting BTC fee in db...");
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &fee, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| BtcDbUtils::new(db).put_btc_fee_in_db(fee))
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"sucess":true,"new_btc_fee":fee}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{get_test_database, DUMMY_DEBUG_COMMAND_SIGNATURE};

    #[test]
    fn should_put_btc_fee_in_db() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let fee = 6;
        db_utils.put_btc_fee_in_db(fee).unwrap();
        assert_eq!(db_utils.get_btc_fee_from_db().unwrap(), fee);
        let new_fee = 4;
        debug_set_btc_fee(&db, new_fee, &CoreType::BtcOnInt, &DUMMY_DEBUG_COMMAND_SIGNATURE).unwrap();
        assert_eq!(db_utils.get_btc_fee_from_db().unwrap(), new_fee);
    }
}
