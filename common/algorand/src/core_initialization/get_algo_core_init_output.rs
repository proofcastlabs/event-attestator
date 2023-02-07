use common::{constants::ZERO_CONFS_WARNING, traits::DatabaseInterface, types::Result};
use serde::{Deserialize, Serialize};

use crate::AlgoDbUtils;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlgoInitializationOutput {
    algo_address: String,
    algo_latest_block_num: u64,
    number_of_confirmations: String,
}

impl AlgoInitializationOutput {
    pub fn new<D: DatabaseInterface>(algo_db_utils: &AlgoDbUtils<D>) -> Result<Self> {
        let number_of_confirmations = algo_db_utils.get_canon_to_tip_length()?;
        Ok(Self {
            algo_latest_block_num: algo_db_utils.get_latest_block_number()?,
            algo_address: algo_db_utils.get_redeem_address()?.to_string(),
            number_of_confirmations: if number_of_confirmations == 0 {
                ZERO_CONFS_WARNING.to_string()
            } else {
                number_of_confirmations.to_string()
            },
        })
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;
    use rust_algorand::AlgorandAddress;

    use super::*;
    use crate::test_utils::get_sample_submission_material_n;

    #[test]
    fn should_get_init_output() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_material = get_sample_submission_material_n(0);
        let address = AlgorandAddress::default();
        db_utils
            .put_latest_submission_material_in_db(&submission_material)
            .unwrap();
        db_utils.put_redeem_address_in_db(&address).unwrap();
        let number_of_confirmations = 1337;
        db_utils.put_canon_to_tip_length_in_db(number_of_confirmations).unwrap();
        let result = AlgoInitializationOutput::new(&db_utils).unwrap();
        let expected_result = AlgoInitializationOutput {
            algo_address: address.to_string(),
            algo_latest_block_num: submission_material.block.round(),
            number_of_confirmations: number_of_confirmations.to_string(),
        };
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_init_output_with_zero_confs_warning() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let submission_material = get_sample_submission_material_n(0);
        let address = AlgorandAddress::default();
        db_utils
            .put_latest_submission_material_in_db(&submission_material)
            .unwrap();
        db_utils.put_redeem_address_in_db(&address).unwrap();
        let number_of_confirmations = 0;
        db_utils.put_canon_to_tip_length_in_db(number_of_confirmations).unwrap();
        let result = AlgoInitializationOutput::new(&db_utils).unwrap();
        let expected_result = AlgoInitializationOutput {
            algo_address: address.to_string(),
            algo_latest_block_num: submission_material.block.round(),
            number_of_confirmations: ZERO_CONFS_WARNING.to_string(),
        };
        assert_eq!(result, expected_result);
    }
}
