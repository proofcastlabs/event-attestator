use common::{traits::DatabaseInterface, types::Result};

use crate::{maybe_get_new_candidate_block_hash, AlgoState};

pub fn maybe_update_algo_canon_block_hash_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Maybe updating ALGO canon block hash...");
    maybe_get_new_candidate_block_hash(
        &state.algo_db_utils.get_canon_submission_material()?,
        state
            .algo_db_utils
            .maybe_get_new_canon_submission_material_candidate()?,
    )
    .and_then(|maybe_new_canon_block_hash| match maybe_new_canon_block_hash {
        None => {
            info!("✔ No new ALGO canon block candidate found ∴ not updating!");
            Ok(state)
        },
        Some(hash) => {
            info!("✔ New ALGO canon block candidate found ∴ updating...");
            state.algo_db_utils.put_canon_block_hash_in_db(&hash)?;
            Ok(state)
        },
    })
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{test_utils::get_sample_contiguous_submission_material, AlgoDbUtils};

    #[test]
    fn should_not_update_canon_block_hash_if_no_candidate_found() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let contiguous_submission_materials = get_sample_contiguous_submission_material();
        let num_submission_materials = contiguous_submission_materials.len();
        let canon_to_tip_length = num_submission_materials + 1;
        let current_canon_block_hash = contiguous_submission_materials[num_submission_materials - 1]
            .block
            .hash()
            .unwrap();
        let latest_block_hash = current_canon_block_hash;
        let expected_result = current_canon_block_hash;
        contiguous_submission_materials
            .iter()
            .for_each(|material| db_utils.put_algo_submission_material_in_db(material).unwrap());
        db_utils.put_canon_block_hash_in_db(&current_canon_block_hash).unwrap();
        db_utils.put_latest_block_hash_in_db(&latest_block_hash).unwrap();
        db_utils
            .put_canon_to_tip_length_in_db(canon_to_tip_length as u64)
            .unwrap();
        assert_eq!(latest_block_hash, db_utils.get_latest_block_hash().unwrap());
        assert_eq!(current_canon_block_hash, db_utils.get_canon_block_hash().unwrap());
        assert_eq!(canon_to_tip_length as u64, db_utils.get_canon_to_tip_length().unwrap());
        maybe_update_algo_canon_block_hash_and_return_state(AlgoState::init(&db)).unwrap();
        let result = db_utils.get_canon_block_hash().unwrap();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_update_canon_block_hash_if_candidate_block_is_newer_than_current_canon_block() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let contiguous_submission_materials = get_sample_contiguous_submission_material();
        let num_submission_materials = contiguous_submission_materials.len();
        let canon_to_tip_length = 5;
        let current_canon_submission_material =
            contiguous_submission_materials[num_submission_materials - (canon_to_tip_length + 3)].clone();
        let current_canon_block_hash = current_canon_submission_material.block.hash().unwrap();
        let expected_result = contiguous_submission_materials[num_submission_materials - (canon_to_tip_length + 1)]
            .block
            .hash()
            .unwrap();
        contiguous_submission_materials
            .iter()
            .for_each(|material| db_utils.put_algo_submission_material_in_db(material).unwrap());
        let latest_block_hash = contiguous_submission_materials[num_submission_materials - 1]
            .block
            .hash()
            .unwrap();
        db_utils.put_canon_block_hash_in_db(&current_canon_block_hash).unwrap();
        db_utils.put_latest_block_hash_in_db(&latest_block_hash).unwrap();
        db_utils
            .put_canon_to_tip_length_in_db(canon_to_tip_length as u64)
            .unwrap();
        assert_eq!(latest_block_hash, db_utils.get_latest_block_hash().unwrap());
        assert_eq!(current_canon_block_hash, db_utils.get_canon_block_hash().unwrap());
        assert_eq!(canon_to_tip_length as u64, db_utils.get_canon_to_tip_length().unwrap());
        maybe_update_algo_canon_block_hash_and_return_state(AlgoState::init(&db)).unwrap();
        let result = db_utils.get_canon_block_hash().unwrap();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_not_update_canon_block_hash_if_candidate_block_older_than_current_canon_block() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let contiguous_submission_materials = get_sample_contiguous_submission_material();
        let num_submission_materials = contiguous_submission_materials.len();
        let canon_to_tip_length = 5;
        let current_canon_submission_material =
            contiguous_submission_materials[num_submission_materials - canon_to_tip_length].clone();
        let current_canon_block_hash = current_canon_submission_material.block.hash().unwrap();
        let expected_result = current_canon_block_hash;
        contiguous_submission_materials
            .iter()
            .for_each(|block| db_utils.put_algo_submission_material_in_db(block).unwrap());
        let latest_block_hash = contiguous_submission_materials[num_submission_materials - 1]
            .block
            .hash()
            .unwrap();
        db_utils.put_canon_block_hash_in_db(&current_canon_block_hash).unwrap();
        db_utils.put_latest_block_hash_in_db(&latest_block_hash).unwrap();
        db_utils
            .put_canon_to_tip_length_in_db(canon_to_tip_length as u64)
            .unwrap();
        assert_eq!(latest_block_hash, db_utils.get_latest_block_hash().unwrap());
        assert_eq!(current_canon_block_hash, db_utils.get_canon_block_hash().unwrap());
        assert_eq!(canon_to_tip_length as u64, db_utils.get_canon_to_tip_length().unwrap());
        maybe_update_algo_canon_block_hash_and_return_state(AlgoState::init(&db)).unwrap();
        let result = db_utils.get_canon_block_hash().unwrap();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_move_canon_block_correctly() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let contiguous_submission_materials = get_sample_contiguous_submission_material();
        let canon_to_tip_length = 3;
        let starting_canon_block_hash = contiguous_submission_materials[0].block.hash().unwrap();
        db_utils.put_canon_to_tip_length_in_db(canon_to_tip_length).unwrap();
        db_utils.put_canon_block_hash_in_db(&starting_canon_block_hash).unwrap();
        contiguous_submission_materials
            .iter()
            .enumerate()
            .for_each(|(i, material)| {
                db_utils.put_algo_submission_material_in_db(material).unwrap();
                db_utils
                    .put_latest_block_hash_in_db(&material.block.hash().unwrap())
                    .unwrap();
                maybe_update_algo_canon_block_hash_and_return_state(AlgoState::init(&db)).unwrap();
                let result = db_utils.get_canon_block_hash().unwrap();
                if i <= canon_to_tip_length as usize {
                    assert_eq!(result, starting_canon_block_hash);
                } else {
                    assert_eq!(
                        result,
                        contiguous_submission_materials[i - canon_to_tip_length as usize]
                            .block
                            .hash()
                            .unwrap()
                    );
                }
            })
    }
}
