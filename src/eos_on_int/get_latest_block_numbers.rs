use serde::{Deserialize, Serialize};

use crate::{
    chains::{
        eos::eos_database_utils::EosDbUtils,
        eth::eth_database_utils::{EthDbUtils, EthDbUtilsExt},
    },
    eos_on_int::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct BlockNumbers {
    int_latest_block_number: usize,
    eos_latest_block_number: u64,
}

/// # Get Latest Block Numbers
///
/// This function returns a JSON containing the last processed block number of each of the
/// blockchains this instance manages.
pub fn get_latest_block_numbers<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting latest block numbers...");
    let int_db_utils = EthDbUtils::new(db);
    let eos_db_utils = EosDbUtils::new(db);
    check_core_is_initialized(&int_db_utils, &eos_db_utils).and_then(|_| {
        Ok(serde_json::to_string(&BlockNumbers {
            int_latest_block_number: int_db_utils.get_latest_eth_block_number()?,
            eos_latest_block_number: eos_db_utils.get_latest_eos_block_number()?,
        })?)
    })
}
