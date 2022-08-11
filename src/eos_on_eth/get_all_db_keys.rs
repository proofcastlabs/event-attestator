use serde_json::json;

use crate::{
    chains::{eos::eos_database_utils::EosDatabaseKeysJson, eth::eth_database_utils::EthDatabaseKeysJson},
    constants::DB_KEY_PREFIX,
    debug_mode::DEBUG_SIGNATORIES_DB_KEY,
    dictionaries::dictionary_constants::EOS_ETH_DICTIONARY_KEY,
    types::Result,
};

/// # Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn get_all_db_keys() -> Result<String> {
    Ok(json!({
        "eth": EthDatabaseKeysJson::new(),
        "eos": EosDatabaseKeysJson::new(),
        "db-key-prefix": DB_KEY_PREFIX.to_string(),
        "dictionary:": hex::encode(EOS_ETH_DICTIONARY_KEY.to_vec()),
        "debug_signatories": format!("0x{}", hex::encode(&*DEBUG_SIGNATORIES_DB_KEY)),
    })
    .to_string())
}
