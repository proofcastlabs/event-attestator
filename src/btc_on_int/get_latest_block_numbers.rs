use serde::{Deserialize, Serialize};

use crate::{
    btc_on_int::check_core_is_initialized::check_core_is_initialized,
    chains::{
        btc::btc_database_utils::BtcDbUtils,
        eth::eth_database_utils::{EthDbUtils, EthDbUtilsExt},
    },
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct BlockNumbers {
    btc_latest_block_number: u64,
    int_latest_block_number: usize,
}

/// # Get Latest Block Numbers
///
/// This function returns a JSON containing the last processed block number of each of the
/// blockchains this instance manages.
pub fn get_latest_block_numbers<D: DatabaseInterface>(db: D) -> Result<String> {
    info!("✔ Getting latest block numbers...");
    let eth_db_utils = EthDbUtils::new(&db);
    let btc_db_utils = BtcDbUtils::new(&db);
    check_core_is_initialized(&eth_db_utils, &btc_db_utils).and_then(|_| {
        Ok(serde_json::to_string(&BlockNumbers {
            btc_latest_block_number: btc_db_utils.get_latest_btc_block_number()?,
            int_latest_block_number: eth_db_utils.get_latest_eth_block_number()?,
        })?)
    })
}
