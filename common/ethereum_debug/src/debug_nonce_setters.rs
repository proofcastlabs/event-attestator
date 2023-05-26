use common::{
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use common_eth::{EthDbUtils, EthDbUtilsExt, EvmDbUtils};
use function_name::named;
use serde_json::json;

#[named]
fn debug_set_account_nonce<D: DatabaseInterface>(
    db: &D,
    new_nonce: u64,
    is_for_eth: bool,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!(
        "✔ Debug setting {} account nonce...",
        if is_for_eth { "ETH" } else { "EVM" }
    );
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &new_nonce, &is_for_eth, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
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

#[named]
fn debug_set_any_sender_nonce<D: DatabaseInterface>(
    db: &D,
    new_nonce: u64,
    is_for_eth: bool,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &new_nonce, &is_for_eth, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
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

/// # Debug Set ETH Account Nonce
///
/// This function sets the ETH account nonce to the passed in value in the encryped database.
pub fn debug_set_eth_account_nonce<D: DatabaseInterface>(
    db: &D,
    new_nonce: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug setting ETH account nonce...");
    debug_set_account_nonce(db, new_nonce, true, core_type, signature)
}

/// # Debug Set EVM Account Nonce
///
/// This function sets the EVM account nonce to the passed in value in the encryped database.
pub fn debug_set_evm_account_nonce<D: DatabaseInterface>(
    db: &D,
    new_nonce: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug setting EVM account nonce...");
    debug_set_account_nonce(db, new_nonce, false, core_type, signature)
}

/// # Debug Set ETH AnySender Nonce
///
/// This function sets the ETH AnySender nonce to the passed in value in the encryped database.
pub fn debug_set_eth_any_sender_nonce<D: DatabaseInterface>(
    db: &D,
    new_nonce: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug setting ETH AnySender nonce...");
    debug_set_any_sender_nonce(db, new_nonce, true, core_type, signature)
}

/// # Debug Set EVM AnySender Nonce
///
/// This function sets the EVM AnySender nonce to the passed in value in the encryped database.
pub fn debug_set_evm_any_sender_nonce<D: DatabaseInterface>(
    db: &D,
    new_nonce: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug setting EVM AnySender nonce...");
    debug_set_any_sender_nonce(db, new_nonce, false, core_type, signature)
}

#[cfg(test)]
mod tests {
    use common::{
        errors::AppError,
        test_utils::{get_test_database, DUMMY_DEBUG_COMMAND_SIGNATURE},
    };

    use super::*;

    #[test]
    fn should_set_eth_account_nonce() {
        let db = get_test_database();
        let db_utils = EthDbUtils::new(&db);
        let nonce = 6;
        let is_for_eth = true;
        db_utils.put_eth_account_nonce_in_db(nonce).unwrap();
        assert_eq!(db_utils.get_eth_account_nonce_from_db().unwrap(), nonce);
        let new_nonce = 4;
        debug_set_account_nonce(
            &db,
            new_nonce,
            is_for_eth,
            &CoreType::BtcOnInt,
            DUMMY_DEBUG_COMMAND_SIGNATURE,
        )
        .unwrap();
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
        debug_set_any_sender_nonce(
            &db,
            new_nonce,
            is_for_eth,
            &CoreType::BtcOnInt,
            DUMMY_DEBUG_COMMAND_SIGNATURE,
        )
        .unwrap();
        assert_eq!(db_utils.get_any_sender_nonce_from_db().unwrap(), new_nonce);
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
