use crate::{
    chains::algo::{
        algo_constants::ALGO_CORE_IS_INITIALIZED_JSON,
        algo_database_transactions::{
            end_algo_db_transaction_and_return_state,
            start_algo_db_transaction_and_return_state,
        },
        algo_state::AlgoState,
        core_initialization::{
            get_algo_core_init_output::AlgoInitializationOutput,
            initialize_algo_core::initialize_algo_core,
        },
    },
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
};

/// # Maybe Initialize ALGO Core
///
/// This function first checks to see if the ALGO core has already been initialized, and initializes
/// it if not. The initialization procedure takes as its input a valid ALGO block JSON of the
/// format:
///
/// ```no_compile
/// {
///   'block': <algo-block>,
/// }
/// ```
pub fn maybe_initialize_algo_core<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    genesis_id: &str,
    fee: u64,
    canon_to_tip_length: u64,
    app_id: u64,
) -> Result<String> {
    if CoreType::host_core_is_initialized(db) {
        Ok(ALGO_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        let is_native = false;
        start_algo_db_transaction_and_return_state(AlgoState::init_with_empty_dictionary(db))
            .and_then(|state| {
                initialize_algo_core(
                    state,
                    block_json,
                    fee,
                    canon_to_tip_length,
                    genesis_id,
                    app_id,
                    is_native,
                )
            })
            .and_then(end_algo_db_transaction_and_return_state)
            .and_then(|state| AlgoInitializationOutput::new(&state.algo_db_utils))
            .and_then(|output| output.to_string())
    }
}

#[cfg(test)]
mod tests {
    use rust_algorand::{AlgorandAppId, AlgorandHash, MicroAlgos};

    use super::*;
    use crate::{
        chains::algo::{algo_database_utils::AlgoDbUtils, test_utils::get_sample_submission_material_n},
        test_utils::get_test_database,
    };

    #[test]
    fn should_maybe_init_algo_core() {
        let fee = 1337;
        let app_id = 666;
        let fee_in_micro_algos = MicroAlgos::new(fee);
        let canon_to_tip_length = 3;
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_material = get_sample_submission_material_n(0);
        let hash = submission_material.block.hash().unwrap();
        let genesis_id = "mainnet-v1.0";
        let block_json_string = submission_material.to_string();
        let result =
            maybe_initialize_algo_core(&db, &block_json_string, genesis_id, fee, canon_to_tip_length, app_id).unwrap();
        let expected_result = AlgoInitializationOutput::new(&db_utils).unwrap().to_string().unwrap();
        assert_eq!(result, expected_result);
        assert!(db_utils.get_algo_private_key().is_ok());
        assert_eq!(db_utils.get_algo_fee().unwrap(), fee_in_micro_algos);
        assert_eq!(db_utils.get_algo_account_nonce().unwrap(), 0);
        assert_eq!(db_utils.get_algo_app_id().unwrap(), AlgorandAppId::new(app_id));
        assert_eq!(db_utils.get_tail_block_hash().unwrap(), hash);
        assert_eq!(
            db_utils.get_genesis_hash().unwrap(),
            AlgorandHash::from_genesis_id(genesis_id).unwrap()
        );
        assert_eq!(db_utils.get_canon_block_hash().unwrap(), hash);
        assert_eq!(db_utils.get_anchor_block_hash().unwrap(), hash);
        assert_eq!(db_utils.get_latest_block_hash().unwrap(), hash);
        assert_eq!(db_utils.get_canon_to_tip_length().unwrap(), canon_to_tip_length);
        assert_eq!(
            db_utils.get_latest_submission_material().unwrap().block.transactions,
            None
        );
        assert_eq!(
            submission_material.block.block_header,
            db_utils.get_latest_submission_material().unwrap().block.block_header,
        );
        assert_eq!(
            db_utils.get_redeem_address().unwrap(),
            db_utils.get_algo_private_key().unwrap().to_address().unwrap()
        );
    }

    #[test]
    fn should_not_init_algo_core_twice() {
        let fee = 1337;
        let canon_to_tip_length = 3;
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let block = get_sample_submission_material_n(0);
        let app_id = 666;
        let genesis_id = "mainnet-v1.0";
        let block_json_string = block.to_string();
        let result_1 =
            maybe_initialize_algo_core(&db, &block_json_string, genesis_id, fee, canon_to_tip_length, app_id).unwrap();
        let result_2 =
            maybe_initialize_algo_core(&db, &block_json_string, genesis_id, fee, canon_to_tip_length, app_id).unwrap();
        let expected_result_1 = AlgoInitializationOutput::new(&db_utils).unwrap().to_string().unwrap();
        let expected_result_2 = ALGO_CORE_IS_INITIALIZED_JSON.to_string();
        assert_eq!(result_1, expected_result_1);
        assert_eq!(result_2, expected_result_2);
    }
}
