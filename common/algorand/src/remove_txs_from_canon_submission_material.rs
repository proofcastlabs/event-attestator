use common::{traits::DatabaseInterface, types::Result};

use crate::AlgoState;

pub fn maybe_remove_txs_from_algo_canon_submission_material_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Removing txs from ALGO canon submission material...");
    state
        .algo_db_utils
        .get_canon_submission_material()
        .map(|material| material.remove_txs())
        .map(|material| state.algo_db_utils.update_canon_submission_material_in_db(&material))
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{test_utils::get_sample_submission_material_n, AlgoDbUtils};

    #[test]
    fn should_remove_all_receipts_from_block() {
        let submission_material = get_sample_submission_material_n(0);
        let num_txs_before = submission_material.clone().block.transactions.unwrap().len();
        assert!(num_txs_before > 0);
        let submission_material_after = submission_material.remove_txs();
        assert!(submission_material_after.block.transactions.is_none());
    }

    #[test]
    fn should_remove_receipts_from_canon_block() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let state = AlgoState::init_with_empty_dictionary(&db);
        let canon_submission_material = get_sample_submission_material_n(0);
        db_utils
            .put_canon_submission_material_in_db(&canon_submission_material)
            .unwrap();
        let canon_submission_material_from_db_before = db_utils.get_canon_submission_material().unwrap();
        assert!(canon_submission_material_from_db_before.block.transactions.is_some());
        maybe_remove_txs_from_algo_canon_submission_material_and_return_state(state).unwrap();
        let canon_submission_material_from_db_after = db_utils.get_canon_submission_material().unwrap();
        assert!(canon_submission_material_from_db_after.block.transactions.is_none());
    }
}
