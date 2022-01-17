use serde::{Deserialize, Serialize};

use crate::{
    chains::{
        algo::algo_database_utils::AlgoDbUtils,
        eth::eth_database_utils::{EthDbUtils, EthDbUtilsExt},
    },
    int_on_algo::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct BlockNumbers {
    algo_latest_block_number: u64,
    int_latest_block_number: usize,
}

/// # Get Latest Block Numbers
///
/// This function returns a JSON containing the last processed block number of each of the
/// blockchains this instance manages.
pub fn get_latest_block_numbers<D: DatabaseInterface>(db: D) -> Result<String> {
    info!("✔ Getting latest `INT-on-ALGO` block numbers...");
    let int_db_utils = EthDbUtils::new(&db);
    let algo_db_utils = AlgoDbUtils::new(&db);
    check_core_is_initialized(&int_db_utils, &algo_db_utils).and_then(|_| {
        Ok(serde_json::to_string(&BlockNumbers {
            int_latest_block_number: int_db_utils.get_latest_eth_block_number()?,
            algo_latest_block_number: algo_db_utils.get_latest_block_number()?,
        })?)
    })
}
