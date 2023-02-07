use common::{traits::DatabaseInterface, types::Result};
use rust_algorand::AlgorandHash;

use crate::{AlgoDbUtils, AlgoState, AlgoSubmissionMaterial};

fn recursively_remove_parent_submission_materials_if_not_anchor_block<D: DatabaseInterface>(
    db_utils: &AlgoDbUtils<D>,
    anchor_hash: &AlgorandHash,
    submission_material_whose_parents_to_be_removed: &AlgoSubmissionMaterial,
) -> Result<()> {
    info!("✔ Recursively removing old ALGO block(s)...");
    match db_utils.get_submission_material(
        &submission_material_whose_parents_to_be_removed
            .block
            .get_previous_block_hash()?,
    ) {
        Err(_) => {
            info!("✔ No block found ∵ doing nothing!");
            Ok(())
        },
        Ok(parent_submission_material) => {
            info!("✔ Previous block found, checking if it's the anchor block...");
            let parent_hash = parent_submission_material.block.hash()?;
            if anchor_hash == &parent_hash {
                info!("✔ Block IS the anchor block ∴ not removing it!");
                Ok(())
            } else {
                info!("✔ Block is NOT the anchor block ∴ removing it...");
                db_utils.delete_submission_material_by_hash(&parent_hash).and_then(|_| {
                    recursively_remove_parent_submission_materials_if_not_anchor_block(
                        db_utils,
                        anchor_hash,
                        &parent_submission_material,
                    )
                })
            }
        },
    }
}

pub fn maybe_remove_old_algo_tail_submission_material_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Removing old ALGO tail block...");
    recursively_remove_parent_submission_materials_if_not_anchor_block(
        &state.algo_db_utils,
        &state.algo_db_utils.get_anchor_block_hash()?,
        &state.algo_db_utils.get_tail_submission_material()?,
    )
    .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::test_utils::get_sample_contiguous_submission_material;

    #[test]
    fn should_recursively_remove_parent_submission_materials_if_not_anchor_block() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_materials = get_sample_contiguous_submission_material();
        let num_submission_materials = submission_materials.len();
        submission_materials
            .iter()
            .for_each(|material| db_utils.put_algo_submission_material_in_db(material).unwrap());
        let anchor_submission_material = submission_materials[0].clone();
        let anchor_block_hash = anchor_submission_material.block.hash().unwrap();
        let tail_submission_material = submission_materials[num_submission_materials - 1].clone();
        db_utils
            .put_tail_submission_material_in_db(&tail_submission_material)
            .unwrap();
        db_utils
            .put_anchor_submission_material_in_db(&anchor_submission_material)
            .unwrap();
        recursively_remove_parent_submission_materials_if_not_anchor_block(
            &db_utils,
            &anchor_block_hash,
            &tail_submission_material,
        )
        .unwrap();
        submission_materials
            .iter()
            .map(|material| material.block.hash().unwrap())
            .enumerate()
            .for_each(|(i, hash)| {
                let result = db_utils.get_submission_material(&hash);
                if i == 0 || i == num_submission_materials - 1 {
                    assert!(result.is_ok());
                } else {
                    assert!(result.is_err());
                }
            })
    }
}
