use serde::{Deserialize, Serialize};

use crate::{
    chains::{eos::eos_database_utils::get_latest_eos_block_number, eth::eth_database_utils::EthDatabaseUtils},
    eos_on_eth::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct BlockNumbers {
    eth_latest_block_number: usize,
    eos_latest_block_number: u64,
}

/// # Get Latest Block Numbers
///
/// This function returns a JSON containing the last processed block number of each of the
/// blockchains this instance manages.
pub fn get_latest_block_numbers<D: DatabaseInterface>(db: D) -> Result<String> {
    info!("✔ Getting latest block numbers...");
    let eth_db_utils = EthDatabaseUtils::new_for_eth(&db);
    check_core_is_initialized(&eth_db_utils, &db).and_then(|_| {
        Ok(serde_json::to_string(&BlockNumbers {
            eth_latest_block_number: eth_db_utils.get_latest_eth_block_number()?,
            eos_latest_block_number: get_latest_eos_block_number(&db)?,
        })?)
    })
}
