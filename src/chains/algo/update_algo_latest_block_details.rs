use rust_algorand::AlgorandBlock;

use crate::{chains::algo::algo_state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn add_latest_algo_block_and_return_state<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    info!("✔ Updating latest Algo block details...");
    state
        .algo_db_utils
        .put_latest_block_in_db(&state.get_submitted_algo_block()?)
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::algo::test_utils::get_sample_block_n, test_utils::get_test_database};

    #[test]
    fn should_add_latest_algo_block_and_return_state() {
        let db = get_test_database();
        let state_0 = AlgoState::init(&db);
        let block = get_sample_block_n(0);
        let state_1 = state_0.add_submitted_algo_block(&block).unwrap();
        let state_2 = add_latest_algo_block_and_return_state(state_1).unwrap();
        let result = state_2.algo_db_utils.get_latest_block().unwrap();
        assert_eq!(result, block);
    }
}
