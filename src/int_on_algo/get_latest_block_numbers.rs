use serde::{Deserialize, Serialize};

use crate::{
    chains::{
        algo::algo_database_utils::AlgoDbUtils,
        eth::eth_database_utils::{EthDbUtils, EthDbUtilsExt},
    },
    core_type::CoreType,
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
pub fn get_latest_block_numbers<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting latest  block numbers...");
    CoreType::check_is_initialized(db).and_then(|_| {
        Ok(serde_json::to_string(&BlockNumbers {
            int_latest_block_number: EthDbUtils::new(db).get_latest_eth_block_number()?,
            algo_latest_block_number: AlgoDbUtils::new(db).get_latest_block_number()?,
        })?)
    })
}
