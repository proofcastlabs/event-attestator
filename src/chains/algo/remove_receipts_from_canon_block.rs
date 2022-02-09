use rust_algorand::AlgorandBlock;

use crate::{chains::algo::algo_state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn remove_all_receipts_from_block(block: &AlgorandBlock) -> AlgorandBlock {
    let mut mutable_block = block.clone();
    mutable_block.transactions = None;
    mutable_block
}

pub fn maybe_remove_receipts_from_algo_canon_block_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Removing receipts from ALGO canon block...");
    state
        .algo_db_utils
        .get_canon_block()
        .map(|block| remove_all_receipts_from_block(&block))
        .map(|block| state.algo_db_utils.put_canon_block_in_db(&block))
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use serde_json::json;

    use super::*;
    use crate::{
        chains::algo::{algo_database_utils::AlgoDbUtils, test_utils::get_sample_block_n},
        dictionaries::evm_algo::EvmAlgoTokenDictionary,
        test_utils::get_test_database,
    };

    #[test]
    fn should_remove_all_receipts_from_block() {
        let block = get_sample_block_n(0);
        let num_txs_before = block.clone().transactions.unwrap().len();
        assert!(num_txs_before > 0);
        let block_after = remove_all_receipts_from_block(&block);
        assert!(block_after.transactions.is_none());
    }

    #[test]
    fn should_remove_receipts_from_canon_block() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let state = AlgoState::init_with_empty_dictionary(&db);
        let canon_block = get_sample_block_n(0);
        db_utils.put_canon_block_in_db(&canon_block).unwrap();
        let canon_block_from_db_before = db_utils.get_canon_block().unwrap();
        assert!(canon_block_from_db_before.transactions.is_some());
        maybe_remove_receipts_from_algo_canon_block_and_return_state(state).unwrap();
        let canon_block_from_db_after = db_utils.get_canon_block().unwrap();
        assert!(canon_block_from_db_after.transactions.is_none());
    }
}
