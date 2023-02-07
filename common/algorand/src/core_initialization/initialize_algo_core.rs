use std::str::FromStr;

use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use rust_algorand::{AlgorandAppId, AlgorandHash, AlgorandKeys, MicroAlgos};

use crate::{
    add_latest_algo_submission_material_to_db_and_return_state,
    remove_irrelevant_txs_from_submission_material_in_state,
    AlgoDbUtils,
    AlgoState,
    AlgoSubmissionMaterial,
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
    app_id: u64,
    is_native: bool,
) -> Result<AlgoState<'a, D>> {
    info!("✔ Initializing ALGO core...");
    if canon_to_tip_length == 0 {
        return Err("Number of confirmations for an Algo core must be >= 1!".into());
    };
    let submission_material = AlgoSubmissionMaterial::from_str(submission_material_str)?;
    let hash = submission_material.block.hash()?;
    state
        .add_algo_submission_material(&submission_material)
        .and_then(remove_irrelevant_txs_from_submission_material_in_state)
        .and_then(add_latest_algo_submission_material_to_db_and_return_state)
        .and_then(|state| {
            let keys = AlgorandKeys::create_random();
            let address = keys.to_address()?;
            if is_native {
                CoreType::initialize_native_core(state.algo_db_utils.get_db())?
            } else {
                CoreType::initialize_host_core(state.algo_db_utils.get_db())?
            };
            state.algo_db_utils.put_algo_account_nonce_in_db(0)?;
            state.algo_db_utils.put_algo_private_key_in_db(&keys)?;
            state.algo_db_utils.put_redeem_address_in_db(&address)?;
            state.algo_db_utils.put_algo_fee_in_db(MicroAlgos::new(fee))?;
            state.algo_db_utils.put_algo_app_id_in_db(&AlgorandAppId::new(app_id))?;
            initialize_algo_chain_db_keys(&state.algo_db_utils, &hash, canon_to_tip_length)?;
            state
                .algo_db_utils
                .put_genesis_hash_in_db(&AlgorandHash::from_genesis_id(genesis_id)?)?;
            Ok(state)
        })
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{test_utils::get_sample_submission_material_n, AlgoDbUtils};

    #[test]
    fn should_init_algo_core() {
        let fee = 1337;
        let fee_in_micro_algos = MicroAlgos::new(fee);
        let canon_to_tip_length = 3;
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let app_id = 666;
        let state = AlgoState::init_with_empty_dictionary(&db);
        let submission_material = get_sample_submission_material_n(0);
        let hash = submission_material.block.hash().unwrap();
        let genesis_id = "mainnet-v1.0";
        let block_json_string = submission_material.to_string();
        let is_native = false;
        initialize_algo_core(
            state,
            &block_json_string,
            fee,
            canon_to_tip_length,
            genesis_id,
            app_id,
            is_native,
        )
        .unwrap();
        assert!(db_utils.get_algo_private_key().is_ok());
        assert_eq!(db_utils.get_algo_fee().unwrap(), fee_in_micro_algos);
        assert_eq!(db_utils.get_algo_account_nonce().unwrap(), 0);
        assert_eq!(db_utils.get_tail_block_hash().unwrap(), hash);
        assert_eq!(db_utils.get_algo_app_id().unwrap(), AlgorandAppId::new(app_id));
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
