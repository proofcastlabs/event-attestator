use serde::{Deserialize, Serialize};

use crate::{
    chains::{btc::btc_database_utils::BtcDbUtils, eos::eos_database_utils::EosDbUtils},
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct BlockNumbers {
    btc_latest_block_number: u64,
    eos_latest_block_number: u64,
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
            eos_latest_block_number: EosDbUtils::new(db).get_latest_eos_block_number()?,
        })?)
    })
}
