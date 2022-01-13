use serde_json::json;

use crate::{
    chains::eth::eth_database_utils::{EthDbUtils, EthDbUtilsExt, EvmDbUtils},
    check_debug_mode::check_debug_mode,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

fn debug_set_account_nonce<D: DatabaseInterface>(db: &D, new_nonce: u64, is_for_eth: bool) -> Result<String> {
    info!(
        "✔ Debug setting {} account nonce...",
        if is_for_eth { "ETH" } else { "EVM" }
    );
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| {
            if is_for_eth {
                EthDbUtils::new(db).put_eth_account_nonce_in_db(new_nonce)
            } else {
                EvmDbUtils::new(db).put_eth_account_nonce_in_db(new_nonce)
            }
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(
            json!({format!("set_{}_account__nonce", if is_for_eth { "eth" } else { "evm" }): true}).to_string(),
        ))
        .map(prepend_debug_output_marker_to_string)
}

fn debug_set_any_sender_nonce<D: DatabaseInterface>(db: &D, new_nonce: u64, is_for_eth: bool) -> Result<String> {
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| {
            if is_for_eth {
                EthDbUtils::new(db).put_any_sender_nonce_in_db(new_nonce)
            } else {
                EvmDbUtils::new(db).put_any_sender_nonce_in_db(new_nonce)
            }
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(
            json!({format!("set_{}_any_sender__nonce", if is_for_eth { "eth" } else { "evm" }): true}).to_string(),
        ))
        .map(prepend_debug_output_marker_to_string)
}

fn debug_set_gas_price_in_db<D: DatabaseInterface>(db: &D, gas_price: u64, is_for_eth: bool) -> Result<String> {
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| {
            if is_for_eth {
                EthDbUtils::new(db).put_eth_gas_price_in_db(gas_price)
            } else {
                EvmDbUtils::new(db).put_eth_gas_price_in_db(gas_price)
            }
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(
            json!({"sucess":true,format!("new_{}_gas_price", if is_for_eth { "eth" } else { "evm" }):gas_price})
                .to_string(),
        ))
        .map(prepend_debug_output_marker_to_string)
}

pub fn check_custom_nonce<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E, custom_nonce: u64) -> Result<u64> {
    db_utils.get_eth_account_nonce_from_db().and_then(|account_nonce| {
        if custom_nonce >= account_nonce {
            Err(format!(
                "Cannot use custom nonce of {} ∵ it's > account nonce of {}!",
                custom_nonce, account_nonce,
            )
            .into())
        } else {
            Ok(custom_nonce)
        }
    })
}

/// Debug Set ETH Gas Price
///
/// This function sets the ETH gas price to use when making ETH transactions. It's unit is `Wei`.
pub fn debug_set_eth_gas_price_in_db<D: DatabaseInterface>(db: &D, gas_price: u64) -> Result<String> {
    info!("✔ Setting ETH gas price in db...");
    debug_set_gas_price_in_db(db, gas_price, true)
}

/// Debug Set EVM Gas Price
///
/// This function sets the EVM gas price to use when making EVM transactions. It's unit is `Wei`.
pub fn debug_set_evm_gas_price_in_db<D: DatabaseInterface>(db: &D, gas_price: u64) -> Result<String> {
    info!("✔ Setting EVM gas price in db...");
    debug_set_gas_price_in_db(db, gas_price, false)
}

/// # Debug Set ETH Account Nonce
///
/// This function sets the ETH account nonce to the passed in value in the encryped database.
pub fn debug_set_eth_account_nonce<D: DatabaseInterface>(db: &D, new_nonce: u64) -> Result<String> {
    info!("✔ Debug setting ETH account nonce...");
    debug_set_account_nonce(db, new_nonce, true)
}

/// # Debug Set EVM Account Nonce
///
/// This function sets the EVM account nonce to the passed in value in the encryped database.
pub fn debug_set_evm_account_nonce<D: DatabaseInterface>(db: &D, new_nonce: u64) -> Result<String> {
    info!("✔ Debug setting EVM account nonce...");
    debug_set_account_nonce(db, new_nonce, false)
}

/// # Debug Set ETH AnySender Nonce
///
/// This function sets the ETH AnySender nonce to the passed in value in the encryped database.
pub fn debug_set_eth_any_sender_nonce<D: DatabaseInterface>(db: &D, new_nonce: u64) -> Result<String> {
    info!("✔ Debug setting ETH AnySender nonce...");
    debug_set_any_sender_nonce(db, new_nonce, true)
}

/// # Debug Set EVM AnySender Nonce
///
/// This function sets the EVM AnySender nonce to the passed in value in the encryped database.
pub fn debug_set_evm_any_sender_nonce<D: DatabaseInterface>(db: &D, new_nonce: u64) -> Result<String> {
    info!("✔ Debug setting EVM AnySender nonce...");
    debug_set_any_sender_nonce(db, new_nonce, false)
}

#[cfg(all(test, feature = "debug"))]
mod tests {
    use super::*;
    use crate::{errors::AppError, test_utils::get_test_database};

    #[test]
    fn should_set_eth_account_nonce() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let nonce = 6;
        let is_for_eth = true;
        db_utils.put_eth_account_nonce_in_db(nonce).unwrap();
        assert_eq!(db_utils.get_eth_account_nonce_from_db().unwrap(), nonce);
        let new_nonce = 4;
        debug_set_account_nonce(&db, new_nonce, is_for_eth).unwrap();
        assert_eq!(db_utils.get_eth_account_nonce_from_db().unwrap(), new_nonce);
    }

    #[test]
    fn should_set_eth_any_sender_nonce() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let nonce = 6;
        db_utils.put_any_sender_nonce_in_db(nonce).unwrap();
        assert_eq!(db_utils.get_any_sender_nonce_from_db().unwrap(), nonce);
        let new_nonce = 4;
        let is_for_eth = true;
        debug_set_any_sender_nonce(&db, new_nonce, is_for_eth).unwrap();
        assert_eq!(db_utils.get_any_sender_nonce_from_db().unwrap(), new_nonce);
    }

    #[test]
    fn should_set_eth_gas_price_in_db() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let gas_price = 6;
        db_utils.put_eth_gas_price_in_db(gas_price).unwrap();
        assert_eq!(db_utils.get_eth_gas_price_from_db().unwrap(), gas_price);
        let new_gas_price = 4;
        let is_for_eth = true;
        debug_set_gas_price_in_db(&db, new_gas_price, is_for_eth).unwrap();
        assert_eq!(db_utils.get_eth_gas_price_from_db().unwrap(), new_gas_price);
    }

    #[test]
    fn should_check_custom_nonce() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let account_nonce = 10;
        let custom_nonce = account_nonce - 1;
        db_utils.put_eth_account_nonce_in_db(account_nonce).unwrap();
        let result = check_custom_nonce(&db_utils, custom_nonce).unwrap();
        assert_eq!(result, custom_nonce)
    }

    #[test]
    fn should_not_pass_custom_nonce_check_if_greater_than_account_nonce() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let account_nonce = 10;
        let custom_nonce = account_nonce + 1;
        let expected_error = format!(
            "Cannot use custom nonce of {} ∵ it's > account nonce of {}!",
            custom_nonce, account_nonce
        );
        db_utils.put_eth_account_nonce_in_db(account_nonce).unwrap();
        match check_custom_nonce(&db_utils, custom_nonce) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_not_pass_custom_nonce_check_if_equal_to_account_nonce() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let account_nonce = 10;
        let custom_nonce = account_nonce;
        let expected_error = format!(
            "Cannot use custom nonce of {} ∵ it's > account nonce of {}!",
            custom_nonce, account_nonce
        );
        db_utils.put_eth_account_nonce_in_db(account_nonce).unwrap();
        match check_custom_nonce(&db_utils, custom_nonce) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
