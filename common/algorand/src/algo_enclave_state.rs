use common::{constants::ALGO_TAIL_LENGTH, traits::DatabaseInterface, types::Result};
use serde::{Deserialize, Serialize};

use crate::{AlgoDbUtils, ALGO_SAFE_ADDRESS};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlgoEnclaveState {
    algo_fee: u64,
    algo_app_id: String,
    algo_address: String,
    algo_tail_length: u64,
    algo_account_nonce: u64,
    algo_linker_hash: String,
    algo_safe_address: String,
    algo_genesis_hash: String,
    algo_tail_block_number: u64,
    algo_tail_block_hash: String,
    algo_canon_block_number: u64,
    algo_anchor_block_number: u64,
    algo_latest_block_number: u64,
    algo_canon_to_tip_length: u64,
    algo_core_is_validating: bool,
    algo_canon_block_hash: String,
    algo_anchor_block_hash: String,
    algo_latest_block_hash: String,
}

impl AlgoEnclaveState {
    pub fn new<D: DatabaseInterface>(db_utils: &AlgoDbUtils<D>) -> Result<Self> {
        let tail_block = db_utils.get_tail_submission_material()?.block;
        let tail_block_number = tail_block.round();
        let tail_block_hash = tail_block.hash()?.to_string();
        let canon_block = db_utils.get_canon_submission_material()?.block;
        let canon_block_number = canon_block.round();
        let canon_block_hash = canon_block.hash()?.to_string();
        let latest_block = db_utils.get_latest_submission_material()?.block;
        let latest_block_number = latest_block.round();
        let latest_block_hash = latest_block.hash()?.to_string();
        let anchor_block = db_utils.get_anchor_submission_material()?.block;
        let anchor_block_number = anchor_block.round();
        let anchor_block_hash = anchor_block.hash()?.to_string();
        Ok(Self {
            algo_tail_length: ALGO_TAIL_LENGTH,
            algo_tail_block_hash: tail_block_hash,
            algo_canon_block_hash: canon_block_hash,
            algo_anchor_block_hash: anchor_block_hash,
            algo_tail_block_number: tail_block_number,
            algo_latest_block_hash: latest_block_hash,
            algo_canon_block_number: canon_block_number,
            algo_anchor_block_number: anchor_block_number,
            algo_latest_block_number: latest_block_number,
            algo_fee: db_utils.get_algo_fee()?.to_algos(),
            algo_safe_address: ALGO_SAFE_ADDRESS.to_string(),
            algo_app_id: db_utils.get_algo_app_id()?.to_string(),
            algo_account_nonce: db_utils.get_algo_account_nonce()?,
            algo_address: db_utils.get_redeem_address()?.to_string(),
            algo_core_is_validating: !cfg!(feature = "non-validating"),
            algo_genesis_hash: db_utils.get_genesis_hash()?.to_string(),
            algo_canon_to_tip_length: db_utils.get_canon_to_tip_length()?,
            algo_linker_hash: db_utils.get_linker_hash_or_else_genesis_hash()?.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;
    use rust_algorand::{AlgorandHash, MicroAlgos};

    use super::*;
    use crate::{initialize_algo_core, test_utils::get_sample_submission_material_n, AlgoState};

    #[test]
    fn should_get_enclave_state_after_core_is_initialized() {
        let fee = 1337;
        let canon_to_tip_length = 3;
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let state = AlgoState::init_with_empty_dictionary(&db);
        let submission_material = get_sample_submission_material_n(0);
        let block_num = submission_material.block.round();
        let app_id = 666;
        let hash = submission_material.block.hash().unwrap();
        let genesis_id = "mainnet-v1.0";
        let submission_material_json_str = submission_material.to_string();
        let is_native = false;
        initialize_algo_core(
            state,
            &submission_material_json_str,
            fee,
            canon_to_tip_length,
            genesis_id,
            app_id,
            is_native,
        )
        .unwrap();
        let result = AlgoEnclaveState::new(&db_utils).unwrap();
        let expected_result = AlgoEnclaveState {
            algo_account_nonce: 0,
            algo_tail_block_number: block_num,
            algo_app_id: format!("{}", app_id),
            algo_tail_length: ALGO_TAIL_LENGTH,
            algo_canon_block_number: block_num,
            algo_anchor_block_number: block_num,
            algo_latest_block_number: block_num,
            algo_tail_block_hash: hash.to_string(),
            algo_canon_block_hash: hash.to_string(),
            algo_anchor_block_hash: hash.to_string(),
            algo_latest_block_hash: hash.to_string(),
            algo_fee: MicroAlgos::new(fee).to_algos(),
            algo_canon_to_tip_length: canon_to_tip_length,
            algo_safe_address: ALGO_SAFE_ADDRESS.to_string(),
            algo_linker_hash: AlgorandHash::default().to_string(),
            algo_genesis_hash: AlgorandHash::from_genesis_id(genesis_id).unwrap().to_string(),
            algo_core_is_validating: !cfg!(feature = "non-validating"),
            // NOTE: The redeem address is generated randomly on initialization!
            algo_address: db_utils
                .get_algo_private_key()
                .unwrap()
                .to_address()
                .unwrap()
                .to_string(),
        };
        assert_eq!(result.algo_fee, expected_result.algo_fee);
        assert_eq!(result.algo_tail_length, expected_result.algo_tail_length);
        assert_eq!(result.algo_linker_hash, expected_result.algo_linker_hash);
        assert_eq!(result.algo_genesis_hash, expected_result.algo_genesis_hash);
        assert_eq!(result.algo_safe_address, expected_result.algo_safe_address);
        assert_eq!(result.algo_account_nonce, expected_result.algo_account_nonce);
        assert_eq!(result.algo_address, expected_result.algo_address);
        assert_eq!(result.algo_tail_block_hash, expected_result.algo_tail_block_hash);
        assert_eq!(result.algo_canon_block_hash, expected_result.algo_canon_block_hash);
        assert_eq!(result.algo_anchor_block_hash, expected_result.algo_anchor_block_hash);
        assert_eq!(result.algo_latest_block_hash, expected_result.algo_latest_block_hash);
        assert_eq!(result.algo_tail_block_number, expected_result.algo_tail_block_number);
        assert_eq!(result.algo_canon_block_number, expected_result.algo_canon_block_number);
        assert_eq!(
            result.algo_anchor_block_number,
            expected_result.algo_anchor_block_number
        );
        assert_eq!(
            result.algo_latest_block_number,
            expected_result.algo_latest_block_number
        );
        assert_eq!(
            result.algo_canon_to_tip_length,
            expected_result.algo_canon_to_tip_length
        );
        assert_eq!(result, expected_result);
    }
}
