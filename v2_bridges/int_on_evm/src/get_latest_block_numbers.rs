use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_eth::{EthDbUtils, EthDbUtilsExt, EvmDbUtils};
use serde::{Deserialize, Serialize};

use super::constants::CORE_TYPE;

#[derive(Serialize, Deserialize)]
struct BlockNumbers {
    int_latest_block_number: usize,
    evm_latest_block_number: usize,
}

/// # Get Latest Block Numbers
///
/// This function returns a JSON containing the last processed block number of each of the
/// blockchains this instance manages.
pub fn get_latest_block_numbers<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting latest {} block numbers...", CORE_TYPE);
    CoreType::check_is_initialized(db).and_then(|_| {
        Ok(serde_json::to_string(&BlockNumbers {
            int_latest_block_number: EthDbUtils::new(db).get_latest_eth_block_number()?,
            evm_latest_block_number: EvmDbUtils::new(db).get_latest_eth_block_number()?,
        })?)
    })
}
