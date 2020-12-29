use crate::{types::Result, chains::btc::btc_database_utils::get_btc_address_from_db, traits::DatabaseInterface};

pub fn is_btc_enclave_initialized<D: DatabaseInterface>(db: &D) -> bool {
    get_btc_address_from_db(db).is_ok()
}

pub fn check_btc_core_is_initialized<D: DatabaseInterface>(db: &D) -> Result<()> {
    info!("✔ Checking BTC core is initialized...");
    match is_btc_enclave_initialized(db) {
        false => Err("✘ BTC side of core not initialized!".into()),
        true => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::btc::{btc_database_utils::put_btc_address_in_db, btc_test_utils::SAMPLE_TARGET_BTC_ADDRESS},
        test_utils::get_test_database,
    };

    #[test]
    fn should_return_false_if_btc_enc_not_initialized() {
        let db = get_test_database();
        assert!(!is_btc_enclave_initialized(&db));
    }

    #[test]
    fn should_return_true_if_btc_enc_initialized() {
        let db = get_test_database();
        put_btc_address_in_db(&db, &SAMPLE_TARGET_BTC_ADDRESS.to_string()).unwrap();
        assert!(is_btc_enclave_initialized(&db));
    }

    #[test]
    fn should_err_if_btc_core_not_initialized() {
        let db = get_test_database();
        let expected_err = "✘ BTC side of core not initialized!".to_string();
        match check_btc_core_is_initialized(&db)) {
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Ok(_) => panic!("Should not have succeeded!"),
            Err(_) => panic!("Wrong error received!"),

        }
    }

    #[test]
    fn should_be_ok_if_btc_core_initialized() {
        let db = get_test_database();
        put_btc_address_in_db(&db, &SAMPLE_TARGET_BTC_ADDRESS.to_string()).unwrap();
        let result = check_btc_core_is_initialized(&db);
        assert!(result.is_ok());
    }
}
