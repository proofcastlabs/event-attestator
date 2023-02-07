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

/// # Debug Set BTC Account Nonce
///
/// This function set to the given value BTC account nonce in the encryped database.
#[named]
pub fn debug_set_btc_account_nonce<D: DatabaseInterface>(
    db: &D,
    new_nonce: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug setting BTC account nonce...");
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &new_nonce, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| BtcDbUtils::new(db).put_btc_account_nonce_in_db(new_nonce))
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"set_btc_account_nonce":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{get_test_database, DUMMY_DEBUG_COMMAND_SIGNATURE};

    #[test]
    fn should_set_btc_account_nonce() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let nonce = 6;
        db_utils.put_btc_account_nonce_in_db(nonce).unwrap();
        assert_eq!(db_utils.get_btc_account_nonce_from_db().unwrap(), nonce);
        let new_nonce = 4;
        debug_set_btc_account_nonce(&db, new_nonce, &CoreType::BtcOnInt, DUMMY_DEBUG_COMMAND_SIGNATURE).unwrap();
        assert_eq!(db_utils.get_btc_account_nonce_from_db().unwrap(), new_nonce);
    }
}
