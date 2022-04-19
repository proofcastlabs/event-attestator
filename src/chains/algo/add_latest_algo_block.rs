use crate::{chains::algo::algo_state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn add_latest_algo_block_to_db_and_return_state<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    info!("✔ Updating latest Algo submission material details...");
    state
        .algo_db_utils
        .put_latest_submission_material_in_db(&state.get_algo_submission_material()?)
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::algo::test_utils::get_sample_submission_material_n, test_utils::get_test_database};

    #[test]
    fn should_add_latest_algo_block_and_return_state() {
        let db = get_test_database();
        let state_0 = AlgoState::init(&db);
        let submission_material = get_sample_submission_material_n(0);
        let state_1 = state_0.add_algo_submission_material(&submission_material).unwrap();
        let state_2 = add_latest_algo_block_to_db_and_return_state(state_1).unwrap();
        let result = state_2.algo_db_utils.get_latest_submission_material().unwrap();
        assert_eq!(result, submission_material);
    }
}
