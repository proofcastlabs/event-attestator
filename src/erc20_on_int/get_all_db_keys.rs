use serde_json::json;

use crate::{
    chains::eth::eth_database_utils::{EthDatabaseKeysJson, EvmDatabaseKeysJson},
    constants::DB_KEY_PREFIX,
    debug_mode::DEBUG_SIGNATORIES_DB_KEY,
    dictionaries::dictionary_constants::ETH_EVM_DICTIONARY_KEY,
    types::Result,
};

/// # Get All DB Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn get_all_db_keys() -> Result<String> {
    Ok(json!({
        "evm": EvmDatabaseKeysJson::new(),
        "eth": EthDatabaseKeysJson::new(),
        "db_key_prefix": DB_KEY_PREFIX.to_string(),
        "dictionary": hex::encode(ETH_EVM_DICTIONARY_KEY.to_vec()),
        "debug_signatories": format!("0x{}", hex::encode(&*DEBUG_SIGNATORIES_DB_KEY)),
    })
    .to_string())
}
