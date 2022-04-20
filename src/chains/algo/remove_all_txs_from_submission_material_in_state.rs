use crate::{chains::algo::algo_state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn remove_all_txs_from_submission_material_in_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Removing all txs from submission material in state...");
    let mut submission_material = state.get_algo_submission_material()?;
    submission_material.block.transactions = None;
    state.update_algo_submission_material(&submission_material)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::algo::test_utils::get_sample_submission_material_n, test_utils::get_test_database};

    #[test]
    fn should_remove_all_txs_from_block_in_state() {
        let db = get_test_database();
        let submission_material = get_sample_submission_material_n(0);
        let state_1 = AlgoState::init(&db);
        let state_2 = state_1.add_algo_submission_material(&submission_material).unwrap();
        assert!(
            state_2
                .get_algo_submission_material()
                .unwrap()
                .block
                .transactions
                .unwrap()
                .len()
                > 0
        );
        let state_3 = remove_all_txs_from_submission_material_in_state(state_2).unwrap();
        let result = state_3.get_algo_submission_material().unwrap().block.transactions;
        assert!(result.is_none());
    }
}
