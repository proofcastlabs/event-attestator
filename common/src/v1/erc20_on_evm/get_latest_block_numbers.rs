use serde::{Deserialize, Serialize};

use crate::{
    chains::eth::eth_database_utils::{EthDbUtils, EthDbUtilsExt, EvmDbUtils},
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct BlockNumbers {
    eth_latest_block_number: usize,
    evm_latest_block_number: usize,
}

/// # Get Latest Block Numbers
///
/// This function returns a JSON containing the last processed block number of each of the
/// blockchains this instance manages.
pub fn get_latest_block_numbers<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting latest `ERC20-on-EVM` block numbers...");
    CoreType::check_is_initialized(db).and_then(|_| {
        Ok(serde_json::to_string(&BlockNumbers {
            eth_latest_block_number: EthDbUtils::new(db).get_latest_eth_block_number()?,
            evm_latest_block_number: EvmDbUtils::new(db).get_latest_eth_block_number()?,
        })?)
    })
}
