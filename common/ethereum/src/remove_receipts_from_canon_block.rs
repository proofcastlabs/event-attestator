use common::{traits::DatabaseInterface, types::Result};

use crate::{eth_database_utils::EthDbUtilsExt, EthState};

fn remove_receipts_from_canon_block_and_save_in_db<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
) -> Result<()> {
    db_utils
        .get_eth_canon_block_from_db()
        .and_then(|block| db_utils.put_eth_canon_block_in_db(&block.remove_receipts()))
}

pub fn maybe_remove_receipts_from_eth_canon_block_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Removing receipts from ETH canon block...");
    remove_receipts_from_canon_block_and_save_in_db(&state.eth_db_utils).and(Ok(state))
}

pub fn maybe_remove_receipts_from_evm_canon_block_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Removing receipts from EVM canon block...");
    remove_receipts_from_canon_block_and_save_in_db(&state.evm_db_utils).and(Ok(state))
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{eth_database_utils::EthDbUtils, test_utils::get_sample_eth_submission_material};

    #[test]
    fn should_remove_receipts_from_canon_block() {
        let db = get_test_database();
        let canon_block = get_sample_eth_submission_material();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_canon_block_in_db(&canon_block).unwrap();
        let num_receipts_before = eth_db_utils.get_eth_canon_block_from_db().unwrap().receipts.len();
        remove_receipts_from_canon_block_and_save_in_db(&eth_db_utils).unwrap();
        let num_receipts_after = eth_db_utils.get_eth_canon_block_from_db().unwrap().receipts.len();
        assert!(num_receipts_before > 0);
        assert_eq!(num_receipts_after, 0);
    }

    #[test]
    fn should_not_err_if_canon_has_no_receipts() {
        let db = get_test_database();
        let canon_block = get_sample_eth_submission_material().remove_receipts();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_canon_block_in_db(&canon_block).unwrap();
        let num_receipts_before = eth_db_utils.get_eth_canon_block_from_db().unwrap().receipts.len();
        remove_receipts_from_canon_block_and_save_in_db(&eth_db_utils).unwrap();
        let num_receipts_after = eth_db_utils.get_eth_canon_block_from_db().unwrap().receipts.len();
        assert_eq!(num_receipts_before, 0);
        assert_eq!(num_receipts_after, 0);
    }
}
