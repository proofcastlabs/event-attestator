use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_btc::BtcDbUtils;
use common_eth::{EthDbUtils, EthDbUtilsExt};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BlockNumbers {
    btc_latest_block_number: u64,
    int_latest_block_number: usize,
}

/// # Get Latest Block Numbers
///
/// This function returns a JSON containing the last processed block number of each of the
/// blockchains this instance manages.
pub fn get_latest_block_numbers<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting latest block numbers...");
    CoreType::check_is_initialized(db).and_then(|_| {
        Ok(serde_json::to_string(&BlockNumbers {
            btc_latest_block_number: BtcDbUtils::new(db).get_latest_btc_block_number()?,
            int_latest_block_number: EthDbUtils::new(db).get_latest_eth_block_number()?,
        })?)
    })
}
