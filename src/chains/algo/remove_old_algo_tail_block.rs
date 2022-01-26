use rust_algorand::{AlgorandBlock, AlgorandHash};

use crate::{
    chains::algo::{algo_database_utils::AlgoDbUtils, algo_state::AlgoState},
    traits::DatabaseInterface,
    types::Result,
};

pub fn recursively_remove_parent_blocks_if_not_anchor_block<D: DatabaseInterface>(
    db_utils: &AlgoDbUtils<D>,
    anchor_block_hash: &AlgorandHash,
    block_whose_parents_to_be_removed: &AlgorandBlock,
) -> Result<()> {
    info!("✔ Recursively removing old ALGO block(s)...");
    match db_utils.get_block(&block_whose_parents_to_be_removed.get_previous_block_hash()?) {
        Err(_) => {
            info!("✔ No block found ∵ doing nothing!");
            Ok(())
        },
        Ok(parent_block) => {
            info!("✔ Previous block found, checking if it's the anchor block...");
            let parent_block_hash = parent_block.hash()?;
            if anchor_block_hash == &parent_block_hash {
                info!("✔ Block IS the anchor block ∴ not removing it!");
                Ok(())
            } else {
                info!("✔ Block is NOT the anchor block ∴ removing it...");
                db_utils.delete_block_by_block_hash(&parent_block_hash).and_then(|_| {
                    recursively_remove_parent_blocks_if_not_anchor_block(db_utils, anchor_block_hash, &parent_block)
                })
            }
        },
    }
}

fn maybe_remove_old_tail_block_and_return_state<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    info!("✔ Removing old ALGO tail block...");
    recursively_remove_parent_blocks_if_not_anchor_block(
        &state.algo_db_utils,
        &state.algo_db_utils.get_anchor_block_hash()?,
        &state.algo_db_utils.get_tail_block()?,
    )
    .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::algo::test_utils::get_sample_contiguous_blocks, test_utils::get_test_database};

    #[test]
    fn should_recursively_remove_parent_blocks_if_not_anchor_block() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let blocks = get_sample_contiguous_blocks();
        let num_blocks = blocks.len();
        blocks
            .iter()
            .for_each(|block| db_utils.put_block_in_db(&block).unwrap());
        let anchor_block = blocks[0].clone();
        let anchor_block_hash = anchor_block.hash().unwrap();
        let tail_block = blocks[num_blocks - 1].clone();
        db_utils.put_tail_block_in_db(&tail_block).unwrap();
        db_utils.put_anchor_block_in_db(&anchor_block).unwrap();
        recursively_remove_parent_blocks_if_not_anchor_block(&db_utils, &anchor_block_hash, &tail_block).unwrap();
        blocks
            .iter()
            .map(|block| block.hash().unwrap())
            .enumerate()
            .for_each(|(i, hash)| {
                let result = db_utils.get_block(&hash);
                if i == 0 || i == num_blocks - 1 {
                    assert!(result.is_ok());
                } else {
                    assert!(result.is_err());
                }
            })
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::{
            eth_database_utils::EthDbUtils,
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
        let eth_db_utils = EthDbUtils::new(&db);
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
        let eth_db_utils = EthDbUtils::new(&db);
        let anchor_block = blocks[0].clone();
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        let result = is_anchor_block(&eth_db_utils, &anchor_block.get_block_hash().unwrap()).unwrap();
        assert!(result);
    }

    #[test]
    fn should_remove_parent_block_if_parent_is_not_anchor() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let anchor_block = blocks[0].clone();
        let block = blocks[2].clone();
        let parent_block = blocks[1].clone();
        assert_eq!(parent_block.get_block_hash().unwrap(), block.get_parent_hash().unwrap());
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        eth_db_utils.put_eth_submission_material_in_db(&block).unwrap();
        eth_db_utils.put_eth_submission_material_in_db(&parent_block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&parent_block.get_block_hash().unwrap()));
        remove_parents_if_not_anchor(&eth_db_utils, &block).unwrap();
        assert!(!eth_db_utils.eth_block_exists_in_db(&parent_block.get_block_hash().unwrap()));
    }

    #[test]
    fn should_not_remove_parent_block_if_parent_is_anchor() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let blocks = get_sequential_eth_blocks_and_receipts();
        let anchor_block = blocks[0].clone();
        let block = blocks[1].clone();
        assert_eq!(block.get_parent_hash().unwrap(), anchor_block.get_block_hash().unwrap());
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        eth_db_utils.put_eth_submission_material_in_db(&block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&anchor_block.get_block_hash().unwrap()));
        remove_parents_if_not_anchor(&eth_db_utils, &block).unwrap();
        assert!(eth_db_utils.eth_block_exists_in_db(&block.get_block_hash().unwrap()));
    }

    #[test]
    fn should_remove_parent_blocks_recursively_if_not_anchor_blocks() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
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
        remove_parents_if_not_anchor(&eth_db_utils, &tail_block).unwrap();
        blocks
            .iter()
            .for_each(|block| assert!(!eth_db_utils.eth_block_exists_in_db(&block.get_block_hash().unwrap())));
        assert!(eth_db_utils.eth_block_exists_in_db(&tail_block.get_block_hash().unwrap()));
        assert!(eth_db_utils.eth_block_exists_in_db(&anchor_block.get_block_hash().unwrap()));
    }
}
*/
