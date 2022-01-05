use serde::{Deserialize, Serialize};

use crate::{
    chains::eth::eth_database_utils::{EthDbUtils, EthDbUtilsExt, EvmDbUtils},
    erc20_on_int::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct BlockNumbers {
    eth_latest_block_number: usize,
    int_latest_block_number: usize,
}

/// # Get Latest Block Numbers
///
/// This function returns a JSON containing the last processed block number of each of the
/// blockchains this instance manages.
pub fn get_latest_block_numbers<D: DatabaseInterface>(db: D) -> Result<String> {
    info!("✔ Getting latest `erc20-on-int` block numbers...");
    let eth_db_utils = EthDbUtils::new(&db);
    let evm_db_utils = EvmDbUtils::new(&db);
    check_core_is_initialized(&eth_db_utils, &evm_db_utils).and_then(|_| {
        Ok(serde_json::to_string(&BlockNumbers {
            eth_latest_block_number: eth_db_utils.get_latest_eth_block_number()?,
            int_latest_block_number: evm_db_utils.get_latest_eth_block_number()?,
        })?)
    })
}
