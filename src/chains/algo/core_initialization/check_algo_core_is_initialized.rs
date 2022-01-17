use crate::{chains::algo::algo_database_utils::AlgoDbUtils, traits::DatabaseInterface, types::Result};

fn is_algo_core_initialized<D: DatabaseInterface>(db_utils: &AlgoDbUtils<D>) -> bool {
    db_utils.get_redeem_address_from_db().is_ok()
}

pub fn check_algo_core_is_initialized<D: DatabaseInterface>(db_utils: &AlgoDbUtils<D>) -> Result<()> {
    info!("✔ Checking ALGO core is initialized...");
    if is_algo_core_initialized(db_utils) {
        Ok(())
    } else {
        Err("ALGO core not initialized!".into())
    }
}
/*
use crate::{chains::eth::eth_database_utils::EthDbUtilsExt, traits::DatabaseInterface, types::Result};

pub fn is_eth_core_initialized<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> bool {
    db_utils.get_public_eth_address_from_db().is_ok()
}

pub fn check_eth_core_is_initialized<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> Result<()> {
    let core_type = if db_utils.get_is_for_eth() { "ETH" } else { "EVM" };
    info!("✔ Checking {} core is initialized...", core_type);
    if is_eth_core_initialized(db_utils) {
        Ok(())
    } else {
        Err(format!("✘ {} side of core not initialized!", core_type).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::{eth_database_utils::EthDbUtils, eth_test_utils::get_sample_eth_address},
        errors::AppError,
        test_utils::get_test_database,
    };

    #[test]
    fn should_return_false_if_eth_core_not_initialized() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let result = is_eth_core_initialized(&eth_db_utils);
        assert!(!result);
    }

    #[test]
    fn should_return_true_if_eth_core_initialized() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils
            .put_public_eth_address_in_db(&get_sample_eth_address())
            .unwrap();
        let result = is_eth_core_initialized(&eth_db_utils);
        assert!(result);
    }

    #[test]
    fn should_not_err_if_core_initialized() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils
            .put_public_eth_address_in_db(&get_sample_eth_address())
            .unwrap();
        let result = check_eth_core_is_initialized(&eth_db_utils);
        assert!(result.is_ok());
    }

    #[test]
    fn should_err_if_core_not_initialized() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let expected_err = "✘ ETH side of core not initialized!".to_string();
        match check_eth_core_is_initialized(&eth_db_utils) {
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Ok(_) => panic!("Should not have succeeded!"),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
*/
