use common::{traits::DatabaseInterface, types::Result};

use crate::{eth_database_utils::EthDbUtilsExt, eth_submission_material::EthSubmissionMaterial, EthState};

fn does_canon_block_require_updating<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    calculated_canon_block_and_receipts: &EthSubmissionMaterial,
) -> Result<bool> {
    db_utils.get_eth_canon_block_from_db().and_then(|canon_block| {
        Ok(canon_block.get_block_number()? < calculated_canon_block_and_receipts.get_block_number()?)
    })
}

fn maybe_get_nth_ancestor_of_latest_block<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    n: u64,
) -> Result<Option<EthSubmissionMaterial>> {
    info!(
        "✔ Maybe getting ancestor #{} of latest {} block...",
        n,
        if db_utils.get_is_for_eth() { "ETH" } else { "EVM" }
    );
    match db_utils.get_eth_latest_block_from_db() {
        Ok(submission_material) => {
            db_utils.maybe_get_nth_ancestor_eth_submission_material(&submission_material.get_block_hash()?, n)
        },
        Err(_) => Ok(None),
    }
}

pub fn maybe_update_canon_block_hash<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    canon_to_tip_length: u64,
) -> Result<()> {
    match maybe_get_nth_ancestor_of_latest_block(db_utils, canon_to_tip_length)? {
        None => {
            info!("✔ No {}th ancestor block in db yet!", canon_to_tip_length);
            Ok(())
        },
        Some(ancestor_block) => {
            info!("✔ {}th ancestor block found...", canon_to_tip_length);
            match does_canon_block_require_updating(db_utils, &ancestor_block)? {
                true => {
                    info!("✔ Updating canon block...");
                    db_utils.put_eth_canon_block_hash_in_db(&ancestor_block.get_block_hash()?)
                },
                false => {
                    info!("✔ Canon block does not require updating");
                    Ok(())
                },
            }
        },
    }
}

fn maybe_update_canon_block_hash_and_return_state<D: DatabaseInterface>(
    is_for_eth: bool,
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!(
        "✔ Maybe updating {} canon block hash...",
        if is_for_eth { "ETH" } else { "EVM" }
    );
    let canon_to_tip_length = if is_for_eth {
        state.eth_db_utils.get_eth_canon_to_tip_length_from_db()?
    } else {
        state.evm_db_utils.get_eth_canon_to_tip_length_from_db()?
    };
    if is_for_eth {
        maybe_update_canon_block_hash(&state.eth_db_utils, canon_to_tip_length).and(Ok(state))
    } else {
        maybe_update_canon_block_hash(&state.evm_db_utils, canon_to_tip_length).and(Ok(state))
    }
}

pub fn maybe_update_eth_canon_block_hash_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_update_canon_block_hash_and_return_state(true, state)
}

pub fn maybe_update_evm_canon_block_hash_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    maybe_update_canon_block_hash_and_return_state(false, state)
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{
        eth_database_utils::EthDbUtils,
        test_utils::{
            get_eth_canon_block_hash_from_db,
            get_sequential_eth_blocks_and_receipts,
            put_eth_latest_block_in_db,
        },
    };

    #[test]
    fn should_return_true_if_canon_block_requires_updating() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let canon_block = blocks_and_receipts[0].clone();
        let calculated_canon_block = blocks_and_receipts[1].clone();
        eth_db_utils.put_eth_canon_block_in_db(&canon_block).unwrap();
        let result = does_canon_block_require_updating(&eth_db_utils, &calculated_canon_block).unwrap();
        assert!(result);
    }

    #[test]
    fn should_return_false_if_canon_block_does_not_require_updating() {
        let db = get_test_database();
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let canon_block = blocks_and_receipts[0].clone();
        let calculated_canon_block = blocks_and_receipts[0].clone();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_canon_block_in_db(&canon_block).unwrap();
        let eth_db_utils = EthDbUtils::new(&db);
        let result = does_canon_block_require_updating(&eth_db_utils, &calculated_canon_block).unwrap();
        assert!(!result);
    }

    #[test]
    fn should_return_block_if_nth_ancestor_of_latest_block_exists() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let block_1 = blocks_and_receipts[0].clone();
        let block_2 = blocks_and_receipts[1].clone();
        let expected_result = block_1.remove_block();
        eth_db_utils.put_eth_submission_material_in_db(&block_1).unwrap();
        put_eth_latest_block_in_db(&eth_db_utils, &block_2).unwrap();
        let result = maybe_get_nth_ancestor_of_latest_block(&eth_db_utils, 1)
            .unwrap()
            .unwrap();
        assert_eq!(result, expected_result)
    }

    #[test]
    fn should_return_none_if_nth_ancestor_of_latest_block_does_not_exist() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let block_1 = blocks_and_receipts[0].clone();
        put_eth_latest_block_in_db(&eth_db_utils, &block_1).unwrap();
        let result = maybe_get_nth_ancestor_of_latest_block(&eth_db_utils, 1).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn should_maybe_update_canon_block_hash() {
        let db = get_test_database();
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let canon_block = blocks_and_receipts[0].clone();
        let block_1 = blocks_and_receipts[1].clone();
        let latest_block = blocks_and_receipts[2].clone();
        let expected_canon_block_hash = block_1.get_block_hash().unwrap();
        let canon_block_hash_before = canon_block.get_block_hash().unwrap();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_canon_block_in_db(&canon_block).unwrap();
        eth_db_utils.put_eth_submission_material_in_db(&block_1).unwrap();
        put_eth_latest_block_in_db(&eth_db_utils, &latest_block).unwrap();
        maybe_update_canon_block_hash(&eth_db_utils, 1).unwrap();
        let canon_block_hash_after = get_eth_canon_block_hash_from_db(&eth_db_utils).unwrap();
        assert!(canon_block_hash_before != canon_block_hash_after);
        assert_eq!(canon_block_hash_after, expected_canon_block_hash);
    }

    #[test]
    fn should_not_maybe_update_canon_block_hash() {
        let db = get_test_database();
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let canon_block = blocks_and_receipts[0].clone();
        let latest_block = blocks_and_receipts[1].clone();
        let canon_block_hash_before = canon_block.get_block_hash().unwrap();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_canon_block_in_db(&canon_block).unwrap();
        put_eth_latest_block_in_db(&eth_db_utils, &latest_block).unwrap();
        maybe_update_canon_block_hash(&eth_db_utils, 1).unwrap();
        let canon_block_hash_after = get_eth_canon_block_hash_from_db(&eth_db_utils).unwrap();
        assert_eq!(canon_block_hash_before, canon_block_hash_after);
    }
}
