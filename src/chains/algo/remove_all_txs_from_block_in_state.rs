use crate::{chains::algo::algo_state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn remove_all_txs_from_block_in_state<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    info!("✔ Removing all txs from block in state...");
    let mut block = state.get_submitted_algo_block()?;
    block.transactions = None;
    state.update_submitted_block(&block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::algo::test_utils::get_sample_block_n, test_utils::get_test_database};

    #[test]
    fn should_remove_all_txs_from_block_in_state() {
        let db = get_test_database();
        let block = get_sample_block_n(0);
        let state_1 = AlgoState::init(&db);
        let state_2 = state_1.add_submitted_algo_block(&block).unwrap();
        assert!(state_2.get_submitted_algo_block().unwrap().transactions.unwrap().len() > 0);
        let state_3 = remove_all_txs_from_block_in_state(state_2).unwrap();
        let result = state_3.get_submitted_algo_block().unwrap();
        assert!(result.transactions.is_none());
    }
}
