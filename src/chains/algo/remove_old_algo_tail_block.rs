use rust_algorand::{AlgorandBlock, AlgorandHash};

use crate::{
    chains::algo::{algo_database_utils::AlgoDbUtils, algo_state::AlgoState},
    traits::DatabaseInterface,
    types::Result,
};

fn recursively_remove_parent_blocks_if_not_anchor_block<D: DatabaseInterface>(
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

pub fn maybe_remove_old_algo_tail_block_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
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
