use common::types::Result;
use rust_algorand::AlgorandHash;

use crate::AlgoSubmissionMaterial;

pub fn maybe_get_new_candidate_block_hash(
    current_submission_material: &AlgoSubmissionMaterial,
    maybe_candidate_submission_material: Option<AlgoSubmissionMaterial>,
) -> Result<Option<AlgorandHash>> {
    match maybe_candidate_submission_material {
        None => {
            info!("✔ No candidate submission material in db yet ∴ not updating block hash!");
            Ok(None)
        },
        Some(candidate_material) => {
            info!("✔ Candidate submission material found!");
            if current_submission_material.block.round() < candidate_material.block.round() {
                info!("✔ Current submission material IS older than new candidate material, ∴ updating it...");
                Ok(Some(candidate_material.block.hash()?))
            } else {
                info!("✘ Current submission material is NOT older than new candidate material ∴ NOT updating it!");
                Ok(None)
            }
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_contiguous_submission_material;

    #[test]
    fn should_get_candidate_block_hash_if_newer() {
        let submission_materials = get_sample_contiguous_submission_material();
        let current_submission_material = submission_materials[0].clone();
        let candidate_material = submission_materials[1].clone();
        let expected_result = Some(candidate_material.block.hash().unwrap());
        let result =
            maybe_get_new_candidate_block_hash(&current_submission_material, Some(candidate_material)).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_get_candidate_block_hash_if_not_newer() {
        let submission_materials = get_sample_contiguous_submission_material();
        let current_submission_material = submission_materials[1].clone();
        let candidate_material = submission_materials[0].clone();
        let expected_result = None;
        let result =
            maybe_get_new_candidate_block_hash(&current_submission_material, Some(candidate_material)).unwrap();
        assert_eq!(result, expected_result);
    }
}
