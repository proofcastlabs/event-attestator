use crate::{
    chains::eth::{
        eth_database_utils_redux::EthDatabaseUtils,
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn update_latest_block_hash_if_subsequent<D: DatabaseInterface>(
    eth_db_utils: &EthDatabaseUtils<D>,
    maybe_subsequent_submission_material: &EthSubmissionMaterial,
) -> Result<()> {
    info!("✔ Updating latest ETH block hash if subsequent...");
    eth_db_utils
        .get_eth_latest_block_from_db()
        .and_then(|latest_submission_material| latest_submission_material.get_block_number())
        .and_then(|latest_block_number| {
            match latest_block_number + 1 == maybe_subsequent_submission_material.get_block_number()? {
                false => {
                    info!("✔ Block NOT subsequent ∴ NOT updating latest block hash!");
                    Ok(())
                },
                true => {
                    info!("✔ Block IS subsequent ∴ updating latest block hash...",);
                    eth_db_utils
                        .put_eth_latest_block_hash_in_db(&maybe_subsequent_submission_material.get_block_hash()?)
                },
            }
        })
}

pub fn maybe_update_latest_block_hash_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe updating latest ETH block hash if subsequent...");
    update_latest_block_hash_if_subsequent(&state.eth_db_utils, state.get_eth_submission_material()?).and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::{
            eth_constants::ETH_LATEST_BLOCK_HASH_KEY,
            eth_test_utils::{
                get_eth_latest_block_hash_from_db,
                get_sequential_eth_blocks_and_receipts,
                put_eth_latest_block_in_db,
            },
            eth_types::EthHash,
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_update_latest_block_hash_if_subsequent() {
        let db = get_test_database();
        let eth_db_utils = EthDatabaseUtils::new(&db);
        let latest_submission_material = get_sequential_eth_blocks_and_receipts()[0].clone();
        let latest_block_hash_before = latest_submission_material.get_block_hash().unwrap();
        put_eth_latest_block_in_db(&eth_db_utils, &latest_submission_material).unwrap();
        let subsequent_submission_material = get_sequential_eth_blocks_and_receipts()[1].clone();
        let expected_block_hash_after = subsequent_submission_material.get_block_hash().unwrap();
        update_latest_block_hash_if_subsequent(&eth_db_utils, &subsequent_submission_material).unwrap();
        let latest_block_hash_after = get_eth_latest_block_hash_from_db(&eth_db_utils).unwrap();
        assert_ne!(latest_block_hash_before, latest_block_hash_after);
        assert_eq!(latest_block_hash_after, expected_block_hash_after);
    }

    #[test]
    fn should_not_update_latest_block_hash_if_not_subsequent() {
        let db = get_test_database();
        let eth_db_utils = EthDatabaseUtils::new(&db);
        let latest_submission_material = get_sequential_eth_blocks_and_receipts()[0].clone();
        let latest_block_hash_before = latest_submission_material.get_block_hash().unwrap();
        put_eth_latest_block_in_db(&eth_db_utils, &latest_submission_material).unwrap();
        let non_subsequent_submission_material = get_sequential_eth_blocks_and_receipts()[0].clone();
        update_latest_block_hash_if_subsequent(&eth_db_utils, &non_subsequent_submission_material).unwrap();
        let latest_block_hash_after = eth_db_utils
            .get_hash_from_db_via_hash_key(EthHash::from_slice(&ETH_LATEST_BLOCK_HASH_KEY[..]))
            .unwrap()
            .unwrap();
        assert_eq!(latest_block_hash_before, latest_block_hash_after);
    }
}
