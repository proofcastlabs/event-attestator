use std::str::FromStr;

use rust_algorand::{AlgorandBlock, AlgorandHash, AlgorandKeys, MicroAlgos};

use crate::{
    chains::algo::{
        add_latest_algo_submission_material::add_latest_algo_submission_material_to_db_and_return_state,
        algo_database_utils::AlgoDbUtils,
        algo_state::AlgoState,
        algo_submission_material::AlgoSubmissionMaterial,
        remove_irrelevant_txs_from_submission_material_in_state::remove_irrelevant_txs_from_submission_material_in_state,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn initialize_algo_chain_db_keys<D: DatabaseInterface>(
    algo_db_utils: &AlgoDbUtils<D>,
    block_hash: &AlgorandHash,
    canon_to_tip_length: u64,
) -> Result<()> {
    info!("✔ Initializing ALGO chain DB keys...");
    algo_db_utils.put_tail_block_hash_in_db(block_hash)?;
    algo_db_utils.put_canon_block_hash_in_db(block_hash)?;
    algo_db_utils.put_latest_block_hash_in_db(block_hash)?;
    algo_db_utils.put_anchor_block_hash_in_db(block_hash)?;
    algo_db_utils.put_canon_to_tip_length_in_db(canon_to_tip_length)?;
    Ok(())
}

pub fn initialize_algo_core<'a, D: DatabaseInterface>(
    state: AlgoState<'a, D>,
    submission_material_str: &str,
    fee: u64,
    canon_to_tip_length: u64,
    genesis_id: &str,
) -> Result<AlgoState<'a, D>> {
    info!("✔ Initializing ALGO core...");
    let submission_material = AlgoSubmissionMaterial::from_str(submission_material_str)?;
    let hash = submission_material.block.hash()?;
    state
        .add_algo_submission_material(&submission_material)
        .and_then(remove_irrelevant_txs_from_submission_material_in_state)
        .and_then(add_latest_algo_submission_material_to_db_and_return_state)
        .and_then(|state| {
            let keys = AlgorandKeys::create_random();
            let address = keys.to_address()?;
            state.algo_db_utils.put_algo_account_nonce_in_db(0)?;
            state.algo_db_utils.put_algo_private_key_in_db(&keys)?;
            state.algo_db_utils.put_redeem_address_in_db(&address)?;
            state.algo_db_utils.put_algo_fee_in_db(MicroAlgos::new(fee))?;
            initialize_algo_chain_db_keys(&state.algo_db_utils, &hash, canon_to_tip_length)?;
            state
                .algo_db_utils
                .put_genesis_hash_in_db(&AlgorandHash::from_genesis_id(genesis_id)?)?;
            Ok(state)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::algo::{algo_database_utils::AlgoDbUtils, test_utils::get_sample_submission_material_n},
        test_utils::get_test_database,
    };

    #[test]
    fn should_init_algo_core() {
        let fee = 1337;
        let fee_in_micro_algos = MicroAlgos::new(fee);
        let canon_to_tip_length = 3;
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let state = AlgoState::init_with_empty_dictionary(&db);
        let submission_material = get_sample_submission_material_n(0);
        let hash = submission_material.block.hash().unwrap();
        let genesis_id = "mainnet-v1.0";
        let block_json_string = submission_material.to_string();
        initialize_algo_core(state, &block_json_string, fee, canon_to_tip_length, genesis_id).unwrap();
        assert!(db_utils.get_algo_private_key().is_ok());
        assert_eq!(db_utils.get_algo_fee().unwrap(), fee_in_micro_algos);
        assert_eq!(db_utils.get_algo_account_nonce().unwrap(), 0);
        assert_eq!(db_utils.get_tail_block_hash().unwrap(), hash);
        assert_eq!(
            db_utils.get_genesis_hash().unwrap(),
            AlgorandHash::from_genesis_id(genesis_id).unwrap()
        );
        assert_eq!(db_utils.get_canon_block_hash().unwrap(), hash);
        assert_eq!(db_utils.get_anchor_block_hash().unwrap(), hash);
        assert_eq!(db_utils.get_latest_block_hash().unwrap(), hash);
        assert_eq!(
            db_utils.get_latest_submission_material().unwrap().block.transactions,
            None
        );
        assert_eq!(db_utils.get_canon_to_tip_length().unwrap(), canon_to_tip_length);
        assert_eq!(
            db_utils.get_latest_submission_material().unwrap().block.block_header,
            submission_material.block.block_header
        );
        assert_eq!(
            db_utils.get_redeem_address().unwrap(),
            db_utils.get_algo_private_key().unwrap().to_address().unwrap()
        );
    }
}
