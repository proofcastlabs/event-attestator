use crate::{chains::btc::btc_database_utils::BtcDbUtils, traits::DatabaseInterface, types::Result};

pub fn is_btc_core_initialized<D: DatabaseInterface>(db_utils: &BtcDbUtils<D>) -> bool {
    db_utils.get_btc_address_from_db().is_ok()
}

pub fn check_btc_core_is_initialized<D: DatabaseInterface>(db_utils: &BtcDbUtils<D>) -> Result<()> {
    info!("✔ Checking BTC core is initialized...");
    if is_btc_core_initialized(db_utils) {
        Ok(())
    } else {
        Err("✘ BTC side of core not initialized!".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::btc::{btc_database_utils::BtcDbUtils, btc_test_utils::SAMPLE_TARGET_BTC_ADDRESS},
        test_utils::get_test_database,
    };

    #[test]
    fn should_return_false_if_btc_core_not_initialized() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        assert!(!is_btc_core_initialized(&db_utils));
    }

    #[test]
    fn should_return_true_if_btc_core_initialized() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        db_utils.put_btc_address_in_db(&SAMPLE_TARGET_BTC_ADDRESS).unwrap();
        assert!(is_btc_core_initialized(&db_utils));
    }
}
