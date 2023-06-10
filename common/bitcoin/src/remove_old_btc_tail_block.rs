use common::{traits::DatabaseInterface, types::Result};

use crate::{bitcoin_crate_alias::BlockHash, btc_block::BtcBlockInDbFormat, btc_database_utils::BtcDbUtils, BtcState};

fn is_anchor_block<D: DatabaseInterface>(db_utils: &BtcDbUtils<D>, btc_block_hash: &BlockHash) -> Result<bool> {
    match db_utils.get_btc_anchor_block_hash_from_db() {
        Ok(ref hash) => Ok(hash == btc_block_hash),
        _ => Err("✘ No anchor hash found in db!".into()),
    }
}

fn remove_parents_if_not_anchor<D: DatabaseInterface>(
    db_utils: &BtcDbUtils<D>,
    block_whose_parents_to_be_removed: &BtcBlockInDbFormat,
) -> Result<()> {
    match db_utils.get_btc_block_from_db(&block_whose_parents_to_be_removed.prev_blockhash) {
        Err(_) => {
            info!("✔ No block found ∵ doing nothing!");
            Ok(())
        },
        Ok(parent_block) => {
            info!("✔ Block found, checking if it's the anchor block...");
            match is_anchor_block(db_utils, &parent_block.id)? {
                true => {
                    info!("✔ Block IS the anchor block ∴ not removing it!");
                    Ok(())
                },
                false => {
                    info!("✔ Block is NOT the anchor ∴ removing it...");
                    db_utils
                        .get_db()
                        .delete(parent_block.id.to_vec())
                        .and_then(|_| remove_parents_if_not_anchor(db_utils, &parent_block))
                },
            }
        },
    }
}

pub fn maybe_remove_old_btc_tail_block<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    // NOTE: This function is to be called AFTER the tail block has been updated in a submission
    // pipeline. This way, the old tail block will be an ancestor of the current one (except in
    // the case of a fork), and hence why only the ancestor(s) is/are removed by this function,
    // and not the tail block itself.
    info!("✔ Maybe removing old BTC tail block...");
    state
        .btc_db_utils
        .get_btc_tail_block_from_db()
        .and_then(|tail_block| remove_parents_if_not_anchor(&state.btc_db_utils, &tail_block))
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{
        btc_database_utils::BtcDbUtils,
        test_utils::{
            get_sample_sequential_btc_blocks_in_db_format,
            put_btc_anchor_block_in_db,
            put_btc_tail_block_in_db,
        },
    };

    #[test]
    fn should_return_false_block_is_not_anchor_block() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let blocks = get_sample_sequential_btc_blocks_in_db_format();
        let anchor_block = blocks[0].clone();
        let non_anchor_block = blocks[1].clone();
        assert_ne!(anchor_block, non_anchor_block);
        put_btc_anchor_block_in_db(&db, &anchor_block).unwrap();
        let result = is_anchor_block(&db_utils, &non_anchor_block.id).unwrap();
        assert!(!result);
    }

    #[test]
    fn should_return_true_if_block_is_anchor_block() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let blocks = get_sample_sequential_btc_blocks_in_db_format();
        let anchor_block = blocks[0].clone();
        put_btc_anchor_block_in_db(&db, &anchor_block).unwrap();
        let result = is_anchor_block(&db_utils, &anchor_block.id).unwrap();
        assert!(result);
    }

    #[test]
    fn should_remove_parent_block_if_parent_is_not_anchor() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let blocks = get_sample_sequential_btc_blocks_in_db_format();
        let anchor_block = blocks[0].clone();
        let block = blocks[2].clone();
        let parent_block = blocks[1].clone();
        assert_eq!(parent_block.id, block.prev_blockhash);
        put_btc_anchor_block_in_db(&db, &anchor_block).unwrap();
        db_utils.put_btc_block_in_db(&block).unwrap();
        db_utils.put_btc_block_in_db(&parent_block).unwrap();
        assert!(db_utils.btc_block_exists_in_db(&parent_block.id));
        remove_parents_if_not_anchor(&db_utils, &block).unwrap();
        assert!(!db_utils.btc_block_exists_in_db(&parent_block.id));
    }

    #[test]
    fn should_not_remove_parent_block_if_parent_is_anchor() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let blocks = get_sample_sequential_btc_blocks_in_db_format();
        let anchor_block = blocks[0].clone();
        let block = blocks[1].clone();
        assert_eq!(block.prev_blockhash, anchor_block.id);
        put_btc_anchor_block_in_db(&db, &anchor_block).unwrap();
        db_utils.put_btc_block_in_db(&block).unwrap();
        assert!(db_utils.btc_block_exists_in_db(&anchor_block.id));
        remove_parents_if_not_anchor(&db_utils, &block).unwrap();
        assert!(db_utils.btc_block_exists_in_db(&block.id));
    }

    #[test]
    fn should_remove_parent_blocks_recursively_if_not_anchor_blocks() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let all_blocks = get_sample_sequential_btc_blocks_in_db_format();
        let blocks = &all_blocks[1..all_blocks.len() - 1];
        let tail_block = all_blocks[all_blocks.len() - 1].clone();
        let anchor_block = all_blocks[0].clone();
        put_btc_anchor_block_in_db(&db, &anchor_block).unwrap();
        put_btc_tail_block_in_db(&db, &tail_block).unwrap();
        assert!(db_utils.btc_block_exists_in_db(&anchor_block.id));
        blocks
            .iter()
            .try_for_each(|block| db_utils.put_btc_block_in_db(block))
            .unwrap();
        blocks
            .iter()
            .for_each(|block| assert!(db_utils.btc_block_exists_in_db(&block.id)));
        remove_parents_if_not_anchor(&db_utils, &tail_block).unwrap();
        blocks
            .iter()
            .for_each(|block| assert!(!db_utils.btc_block_exists_in_db(&block.id)));
        assert!(db_utils.btc_block_exists_in_db(&tail_block.id));
        assert!(db_utils.btc_block_exists_in_db(&anchor_block.id));
    }
}
