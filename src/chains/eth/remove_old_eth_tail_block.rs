use ethereum_types::H256 as EthHash;

use crate::{
    chains::eth::{
        eth_database_utils::EthDatabaseUtils,
        eth_state::EthState,
        eth_submission_material::EthSubmissionMaterial,
    },
    traits::DatabaseInterface,
    types::Result,
};

fn is_anchor_block<D: DatabaseInterface>(eth_db_utils: &EthDatabaseUtils<D>, eth_block_hash: &EthHash) -> Result<bool> {
    match eth_db_utils.get_eth_anchor_block_hash_from_db() {
        Ok(ref hash) => Ok(hash == eth_block_hash),
        _ => Err("✘ No anchor hash found in db!".into()),
    }
}

pub fn remove_parents_if_not_anchor<D: DatabaseInterface>(
    eth_db_utils: &EthDatabaseUtils<D>,
    db: &D,
    block_whose_parents_to_be_removed: &EthSubmissionMaterial,
) -> Result<()> {
    match eth_db_utils.get_submission_material_from_db(&block_whose_parents_to_be_removed.get_parent_hash()?) {
        Err(_) => {
            info!("✔ No block found ∵ doing nothing!");
            Ok(())
        },
        Ok(parent_block) => {
            info!("✔ Block found, checking if it's the anchor block...");
            match is_anchor_block(eth_db_utils, &parent_block.get_block_hash()?)? {
                true => {
                    info!("✔ Block IS the anchor block ∴ not removing it!");
                    Ok(())
                },
                false => {
                    info!("✔ Block is NOT the anchor ∴ removing it...");
                    // FIXME / TODO: This should be a db util of some sort!
                    db.delete(parent_block.get_block_hash()?.as_bytes().to_vec())
                        .and_then(|_| remove_parents_if_not_anchor(eth_db_utils, db, &parent_block))
                },
            }
        },
    }
}

pub fn maybe_remove_old_eth_tail_block_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe removing old ETH tail block...");
    state
        .eth_db_utils
        .get_eth_tail_block_from_db()
        .and_then(|tail_block| remove_parents_if_not_anchor(&state.eth_db_utils, state.db, &tail_block))
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::{
            eth_database_utils::EthDatabaseUtils,
            eth_test_utils::{
                get_sequential_eth_blocks_and_receipts,
                put_eth_anchor_block_in_db,
                put_eth_tail_block_in_db,
            },
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_return_false_block_is_not_anchor_block() {
        let db = get_test_database();
        let eth_db_utils = EthDatabaseUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let anchor_block = blocks[0].clone();
        let non_anchor_block = blocks[1].clone();
        assert_ne!(anchor_block, non_anchor_block);
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        let result = is_anchor_block(&eth_db_utils, &non_anchor_block.get_block_hash().unwrap()).unwrap();
        assert!(!result);
    }

    #[test]
    fn should_return_true_if_block_is_anchor_block() {
        let db = get_test_database();
        let blocks = get_sequential_eth_blocks_and_receipts();
        let eth_db_utils = EthDatabaseUtils::new(&db);
        let anchor_block = blocks[0].clone();
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        let result = is_anchor_block(&eth_db_utils, &anchor_block.get_block_hash().unwrap()).unwrap();
        assert!(result);
    }

    #[test]
    fn should_remove_parent_block_if_parent_is_not_anchor() {
        let db = get_test_database();
        let eth_db_utils = EthDatabaseUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let anchor_block = blocks[0].clone();
        let block = blocks[2].clone();
        let parent_block = blocks[1].clone();
        assert_eq!(parent_block.get_block_hash().unwrap(), block.get_parent_hash().unwrap());
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        eth_db_utils.put_eth_submission_material_in_db(&block).unwrap();
        eth_db_utils.put_eth_submission_material_in_db(&parent_block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&parent_block.get_block_hash().unwrap()));
        remove_parents_if_not_anchor(&eth_db_utils, &db, &block).unwrap();
        assert!(!eth_db_utils.eth_block_exists_in_db(&parent_block.get_block_hash().unwrap()));
    }

    #[test]
    fn should_not_remove_parent_block_if_parent_is_anchor() {
        let db = get_test_database();
        let eth_db_utils = EthDatabaseUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let anchor_block = blocks[0].clone();
        let block = blocks[1].clone();
        assert_eq!(block.get_parent_hash().unwrap(), anchor_block.get_block_hash().unwrap());
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        eth_db_utils.put_eth_submission_material_in_db(&block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&anchor_block.get_block_hash().unwrap()));
        remove_parents_if_not_anchor(&eth_db_utils, &db, &block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&block.get_block_hash().unwrap()));
    }

    #[test]
    fn should_remove_parent_blocks_recursively_if_not_anchor_blocks() {
        let db = get_test_database();
        let eth_db_utils = EthDatabaseUtils::new(&db);
        let all_blocks = get_sequential_eth_blocks_and_receipts();
        let blocks = &all_blocks[1..all_blocks.len() - 1];
        let tail_block = all_blocks[all_blocks.len() - 1].clone();
        let anchor_block = all_blocks[0].clone();
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        put_eth_tail_block_in_db(&eth_db_utils, &tail_block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&anchor_block.get_block_hash().unwrap()));
        blocks
            .iter()
            .map(|block| eth_db_utils.put_eth_submission_material_in_db(block))
            .collect::<Result<()>>()
            .unwrap();
        blocks
            .iter()
            .for_each(|block| assert!(eth_db_utils.eth_block_exists_in_db(&block.get_block_hash().unwrap())));
        remove_parents_if_not_anchor(&eth_db_utils, &db, &tail_block).unwrap();
        blocks
            .iter()
            .for_each(|block| assert!(!eth_db_utils.eth_block_exists_in_db(&block.get_block_hash().unwrap())));
        assert!(eth_db_utils.eth_block_exists_in_db(&tail_block.get_block_hash().unwrap()));
        assert!(eth_db_utils.eth_block_exists_in_db(&anchor_block.get_block_hash().unwrap()));
    }
}
