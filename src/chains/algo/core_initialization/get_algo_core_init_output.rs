use serde::{Deserialize, Serialize};

use crate::{chains::algo::algo_database_utils::AlgoDbUtils, traits::DatabaseInterface, types::Result};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AlgoInitializationOutput {
    pub algo_address: String,
    pub algo_latest_block_num: u64,
}

impl AlgoInitializationOutput {
    pub fn new<D: DatabaseInterface>(algo_db_utils: &AlgoDbUtils<D>) -> Result<Self> {
        Ok(Self {
            algo_latest_block_num: algo_db_utils.get_latest_block_number()?,
            algo_address: algo_db_utils.get_redeem_address()?.to_string(),
        })
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

#[cfg(test)]
mod tests {
    use rust_algorand::AlgorandAddress;

    use super::*;
    use crate::{chains::algo::test_utils::get_sample_submission_material_n, test_utils::get_test_database};

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
        let result = AlgoInitializationOutput::new(&db_utils).unwrap();
        let expected_result = AlgoInitializationOutput {
            algo_address: address.to_string(),
            algo_latest_block_num: submission_material.block.round(),
        };
        assert_eq!(result, expected_result);
    }
}
