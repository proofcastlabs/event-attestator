use crate::{chains::eth::eth_database_utils::EthDatabaseUtils, traits::DatabaseInterface};

pub fn is_evm_core_initialized<D: DatabaseInterface>(db_utils: &EthDatabaseUtils<D>) -> bool {
    db_utils.get_public_eth_address_from_db().is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::evm::eth_test_utils::get_sample_eth_address, test_utils::get_test_database};

    #[test]
    fn should_return_false_if_eth_core_not_initialized() {
        let db = get_test_database();
        let db_utils = EthDatabaseUtils::new_for_evm(&db);
        let result = is_evm_core_initialized(&db_utils);
        assert!(!result);
    }

    #[test]
    fn should_return_true_if_eth_core_initialized() {
        let db = get_test_database();
        let db_utils = EthDatabaseUtils::new_for_evm(&db);
        db_utils
            .put_public_eth_address_in_db(&get_sample_eth_address())
            .unwrap();
        let result = is_evm_core_initialized(&db_utils);
        assert!(result);
    }
}
