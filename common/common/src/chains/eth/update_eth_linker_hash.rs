use crate::{
    chains::eth::{
        calculate_linker_hash::calculate_linker_hash,
        eth_database_utils::EthDbUtilsExt,
        eth_submission_material::EthSubmissionMaterial,
        eth_types::EthHash,
    },
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

fn get_new_linker_hash<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
    block_hash_to_link_to: &EthHash,
) -> Result<EthHash> {
    info!("✔ Calculating new linker hash...");
    db_utils.get_eth_anchor_block_from_db().and_then(|anchor_block| {
        Ok(calculate_linker_hash(
            *block_hash_to_link_to,
            anchor_block.get_block_hash()?,
            db_utils.get_linker_hash_or_genesis_hash()?,
        ))
    })
}

fn maybe_get_parent_of_eth_tail_block<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
    db_utils: &E,
) -> Result<Option<EthSubmissionMaterial>> {
    info!("✔ Maybe getting parent of tail block from db...");
    db_utils
        .get_eth_tail_block_from_db()
        .and_then(|canon_block| Ok(db_utils.maybe_get_parent_eth_submission_material(&canon_block.get_block_hash()?)))
}

fn maybe_update_linker_hash<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> Result<()> {
    let block_type = if db_utils.get_is_for_eth() { "ETH" } else { "EVM" };
    info!("✔ Maybe updating the {} linker hash...", block_type);
    match maybe_get_parent_of_eth_tail_block(db_utils)? {
        Some(parent_of_eth_tail_block) => {
            info!("✔ Updating {} linker hash...", block_type);
            get_new_linker_hash(db_utils, &parent_of_eth_tail_block.get_block_hash()?)
                .and_then(|linker_hash| db_utils.put_eth_linker_hash_in_db(linker_hash))
                .map(|_| ())
        },
        None => {
            info!("✔ {} tail has no parent in db ∴ NOT updating linker hash", block_type);
            Ok(())
        },
    }
}

pub fn maybe_update_eth_linker_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    maybe_update_linker_hash(&state.eth_db_utils).and(Ok(state))
}

pub fn maybe_update_evm_linker_hash_and_return_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    maybe_update_linker_hash(&state.evm_db_utils).and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::{
            eth_database_utils::EthDbUtils,
            eth_test_utils::{
                get_eth_linker_hash_from_db,
                get_sequential_eth_blocks_and_receipts,
                put_eth_anchor_block_in_db,
                put_eth_tail_block_in_db,
            },
        },
        test_utils::get_test_database,
    };

    #[test]
    fn should_get_parent_of_canon_if_extant() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let canon_block = blocks_and_receipts[5].clone();
        let parent_of_eth_tail_block = blocks_and_receipts[4].clone();
        let expected_result = parent_of_eth_tail_block.remove_block();
        assert!(canon_block.get_parent_hash().unwrap() == parent_of_eth_tail_block.get_block_hash().unwrap());
        put_eth_tail_block_in_db(&eth_db_utils, &canon_block).unwrap();
        eth_db_utils
            .put_eth_submission_material_in_db(&parent_of_eth_tail_block)
            .unwrap();
        let result = maybe_get_parent_of_eth_tail_block(&eth_db_utils).unwrap().unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_not_get_parent_of_canon_if_extant() {
        let db = get_test_database();
        let eth_db_utils = EthDbUtils::new(&db);
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let canon_block = blocks_and_receipts[5].clone();
        put_eth_tail_block_in_db(&eth_db_utils, &canon_block).unwrap();
        let result = maybe_get_parent_of_eth_tail_block(&eth_db_utils).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn should_get_new_linker_hash() {
        let db = get_test_database();
        let expected_result_hex = "5cfaf026b198808363c898b2f7fcada79d88fe163fa6281211956a5431481ecf";
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let block_hash_to_link_to = blocks_and_receipts[5].get_block_hash().unwrap();
        let anchor_block = blocks_and_receipts[1].clone();
        let linker_hash = blocks_and_receipts[3].get_block_hash().unwrap();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_linker_hash_in_db(linker_hash).unwrap();
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        let result = get_new_linker_hash(&eth_db_utils, &block_hash_to_link_to).unwrap();
        let result_hex = hex::encode(result.as_bytes());
        assert_eq!(result_hex, expected_result_hex);
    }

    #[test]
    fn should_maybe_update_linker_hash_if_canon_parent_extant() {
        let db = get_test_database();
        let expected_result_hex = "726d388bff7dd43ccb0f91e995ec83fd56228a3a7cd6f6eae1bc2855c7e942be";
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let linker_hash_before = blocks_and_receipts[9].get_block_hash().unwrap();
        let anchor_block = blocks_and_receipts[1].clone();
        let canon_block = blocks_and_receipts[5].clone();
        let parent_of_eth_tail_block = blocks_and_receipts[4].clone();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_linker_hash_in_db(linker_hash_before).unwrap();
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        put_eth_tail_block_in_db(&eth_db_utils, &canon_block).unwrap();
        eth_db_utils
            .put_eth_submission_material_in_db(&parent_of_eth_tail_block)
            .unwrap();
        maybe_update_linker_hash(&eth_db_utils).unwrap();
        let linker_hash_after = get_eth_linker_hash_from_db(&eth_db_utils).unwrap();
        let result_hex = hex::encode(linker_hash_after.as_bytes());
        assert!(linker_hash_after != linker_hash_before);
        assert_eq!(result_hex, expected_result_hex);
    }

    #[test]
    fn should_not_update_linker_hash_if_canon_parent_not_extant() {
        let db = get_test_database();
        let expected_result_hex = "f8e2c3efa74ff5523bcb26c7088d982c60440a7f8ccc9027c548386853f962df";
        let blocks_and_receipts = get_sequential_eth_blocks_and_receipts();
        let linker_hash_before = blocks_and_receipts[9].get_block_hash().unwrap();
        let anchor_block = blocks_and_receipts[1].clone();
        let canon_block = blocks_and_receipts[5].clone();
        let eth_db_utils = EthDbUtils::new(&db);
        eth_db_utils.put_eth_linker_hash_in_db(linker_hash_before).unwrap();
        put_eth_anchor_block_in_db(&eth_db_utils, &anchor_block).unwrap();
        put_eth_tail_block_in_db(&eth_db_utils, &canon_block).unwrap();
        maybe_update_linker_hash(&eth_db_utils).unwrap();
        let linker_hash_after = get_eth_linker_hash_from_db(&eth_db_utils).unwrap();
        let result_hex = hex::encode(linker_hash_after.as_bytes());
        assert_eq!(linker_hash_after, linker_hash_before);
        assert_eq!(result_hex, expected_result_hex);
    }
}
