use rust_algorand::AlgorandAddress;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::{chains::algo::algo_database_utils::AlgoDbUtils, traits::DatabaseInterface, types::Result};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AlgoInitializationOutput {
    pub algo_address: String,
    pub algo_latest_block_num: u64,
}

impl AlgoInitializationOutput {
    pub fn new<D: DatabaseInterface>(algo_db_utils: &AlgoDbUtils<D>) -> Result<Self> {
        Ok(Self {
            algo_latest_block_num: algo_db_utils.get_latest_algo_block_number()?,
            algo_address: algo_db_utils.get_public_algo_address_from_db()?.to_string(),
        })
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}
