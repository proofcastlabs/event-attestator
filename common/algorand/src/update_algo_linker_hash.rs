use common::{traits::DatabaseInterface, types::Result};
use rust_algorand::AlgorandHash;
use tiny_keccak::{Hasher, Keccak};

use crate::{AlgoDbUtils, AlgoState, AlgoSubmissionMaterial};

fn calculate_linker_hash(
    hash_to_link_to: &AlgorandHash,
    anchor_block_hash: &AlgorandHash,
    current_linker_hash: &AlgorandHash,
) -> Result<AlgorandHash> {
    info!("✔ Calculating new ALGO linker hash...");
    let data = [
        hash_to_link_to.to_bytes(),
        anchor_block_hash.to_bytes(),
        current_linker_hash.to_bytes(),
    ]
    .concat();
    let mut keccak = Keccak::v256();
    let mut hashed = [0u8; 32];
    keccak.update(&data);
    keccak.finalize(&mut hashed);
    Ok(AlgorandHash::from_bytes(&hashed)?)
}

fn maybe_get_parent_of_tail_submission_material<D: DatabaseInterface>(
    db_utils: &AlgoDbUtils<D>,
) -> Result<Option<AlgoSubmissionMaterial>> {
    let current_tails_previous_block_hash = db_utils
        .get_tail_submission_material()?
        .block
        .get_previous_block_hash()?;
    match db_utils.get_submission_material(&current_tails_previous_block_hash) {
        Ok(submission_material) => Ok(Some(submission_material)),
        Err(_) => Ok(None),
    }
}

pub fn maybe_update_algo_linker_hash_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Maybe updating the ALGO linker hash...");
    match maybe_get_parent_of_tail_submission_material(&state.algo_db_utils)? {
        Some(parent_of_tail_submission_material) => {
            info!("✔ Updating ALGO linker hash...");
            let new_linker_hash = calculate_linker_hash(
                &parent_of_tail_submission_material.block.hash()?,
                &state.algo_db_utils.get_anchor_block_hash()?,
                &state.algo_db_utils.get_linker_hash_or_else_genesis_hash()?,
            )?;
            state.algo_db_utils.put_linker_block_hash_in_db(&new_linker_hash)?;
            Ok(state)
        },
        None => {
            info!("✔ ALGO tail block has no parent in db ∴ NOT updating linker hash");
            Ok(state)
        },
    }
}

#[cfg(test)]
mod tests {
    use common::{constants::ALGO_PTOKEN_GENESIS_HASH, test_utils::get_test_database};

    use super::*;
    use crate::{test_utils::get_sample_contiguous_submission_material, AlgoDbUtils};

    #[test]
    fn should_get_parent_of_tail_block_if_extant() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_materials = get_sample_contiguous_submission_material();
        submission_materials
            .iter()
            .for_each(|block| db_utils.put_algo_submission_material_in_db(block).unwrap());
        let tail_submission_material = submission_materials[submission_materials.len() - 1].clone();
        db_utils
            .put_tail_block_hash_in_db(&tail_submission_material.block.hash().unwrap())
            .unwrap();
        let result = maybe_get_parent_of_tail_submission_material(&db_utils).unwrap();
        let expected_result = Some(submission_materials[submission_materials.len() - 2].clone());
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_get_parent_of_tail_block_if_not_extant() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_materials = get_sample_contiguous_submission_material();
        submission_materials
            .iter()
            .for_each(|block| db_utils.put_algo_submission_material_in_db(block).unwrap());
        let tail_submission_material = submission_materials[0].clone();
        db_utils
            .put_tail_block_hash_in_db(&tail_submission_material.block.hash().unwrap())
            .unwrap();
        let result = maybe_get_parent_of_tail_submission_material(&db_utils).unwrap();
        let expected_result = None;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_calculate_algo_linker_hash() {
        let hash_0 = AlgorandHash::from_bytes(&[0u8; 32]).unwrap();
        let hash_1 = AlgorandHash::from_bytes(&[1u8; 32]).unwrap();
        let hash_2 = AlgorandHash::from_bytes(&[2u8; 32]).unwrap();
        let result = calculate_linker_hash(&hash_0, &hash_1, &hash_2).unwrap();
        let expected_result = AlgorandHash::from_bytes(
            &hex::decode("078307a0909a75087ee67b066ae45056a2dfa03f3c60716ba1c270c0aa29c9a4").unwrap(),
        )
        .unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_maybe_update_algo_linker_hash_and_return_state() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_materials = get_sample_contiguous_submission_material();
        submission_materials
            .iter()
            .for_each(|material| db_utils.put_algo_submission_material_in_db(material).unwrap());
        let anchor_submission_material = submission_materials[0].clone();
        db_utils
            .put_anchor_block_hash_in_db(&anchor_submission_material.block.hash().unwrap())
            .unwrap();
        let expected_results = vec![
            ALGO_PTOKEN_GENESIS_HASH.to_string(),
            "EgTJEpxRcIruemOLz9k0toxCXcpPzX07NP85vH0LFA8=".to_string(),
            "+S5iQBhOS1NR2PDfaIrb8H+H4OR0mZkC7MrJwDOC6e0=".to_string(),
            "b/JxzIqk5YX3Z4D1XTy1wr9gwj5SYdjjo0ETRLaJ2O8=".to_string(),
            "J0PZZNGUSrtPMForV3vemmI5HAMQNQfs55IP9EY70is=".to_string(),
            "seB6aKegjM9IqLO28Y5B5dI1P7PrHRPRsl/6PLj0iF0=".to_string(),
            "1sf6FuRpYgF42ByJKPcWjxtWz+Q4uFBcCdkY0GqiQ/M=".to_string(),
            "bsxzdLaZAIhDHa0IMv6EcT+W3twUCKtAMDHYA4aLZG0=".to_string(),
            "aNzu2XsbtBLQi67PxGQZs1u8sl+0+14Ihb9zVZoO6xw=".to_string(),
            "1dSAgIi/zqp13NoXjdMO+Jj+5qWk6M9yAVtiUiXtjYk=".to_string(),
            "vGp+itrYjz+Q74DosZIAJl5c8zKbiMpiy0orlu8kyjM=".to_string(),
        ];
        submission_materials.iter().enumerate().for_each(|(i, material)| {
            db_utils
                .put_tail_block_hash_in_db(&material.block.hash().unwrap())
                .unwrap();
            maybe_update_algo_linker_hash_and_return_state(AlgoState::init(&db)).unwrap();
            let result = db_utils.get_linker_hash_or_else_genesis_hash().unwrap().to_string();
            assert_eq!(result, expected_results[i]);
        })
    }
}
