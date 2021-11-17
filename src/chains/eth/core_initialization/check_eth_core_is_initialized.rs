use crate::{
    chains::eth::eth_database_utils::{EthDbUtils, EthDbUtilsExt},
    traits::DatabaseInterface,
    types::Result,
};

pub fn is_eth_core_initialized<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>) -> bool {
    eth_db_utils.get_public_eth_address_from_db().is_ok()
}

pub fn check_eth_core_is_initialized<D: DatabaseInterface>(
    eth_db_utils: &EthDbUtils<D>,
    is_for_eth: bool,
) -> Result<()> {
    info!("✔ Checking ETH core is initialized...");
    match is_eth_core_initialized(eth_db_utils) {
        false => Err(format!(
            "✘ {} side of core not initialized!",
            if is_for_eth { "ETH" } else { "EVM" }
        )
        .into()),
        true => Ok(()),
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
        let eth_db_utils = EthDbUtils::new_for_eth(&db);
        let result = is_eth_core_initialized(&eth_db_utils);
        assert!(!result);
    }

    #[test]
    fn should_return_true_if_eth_core_initialized() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new_for_eth(&db);
        eth_db_utils
            .put_public_eth_address_in_db(&get_sample_eth_address())
            .unwrap();
        let result = is_eth_core_initialized(&eth_db_utils);
        assert!(result);
    }

    #[test]
    fn should_not_err_if_core_initialized() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new_for_eth(&db);
        eth_db_utils
            .put_public_eth_address_in_db(&get_sample_eth_address())
            .unwrap();
        let is_for_eth = true;
        let result = check_eth_core_is_initialized(&eth_db_utils, is_for_eth);
        assert!(result.is_ok());
    }

    #[test]
    fn should_err_if_core_not_initialized() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new_for_eth(&db);
        let expected_err = "✘ ETH side of core not initialized!".to_string();
        let is_for_eth = true;
        match check_eth_core_is_initialized(&eth_db_utils, is_for_eth) {
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Ok(_) => panic!("Should not have succeeded!"),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
