use serde_json::json;

use crate::{
    chains::{algo::algo_database_utils::AlgoDatabaseKeysJson, eth::eth_database_utils::EthDatabaseKeysJson},
    constants::DB_KEY_PREFIX,
    debug_functions::DEBUG_SIGNATORIES_DB_KEY,
    dictionaries::dictionary_constants::EVM_ALGO_DICTIONARY_KEY,
    types::Result,
};

/// # Debug Get All DB Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    Ok(json!({
        "int": EthDatabaseKeysJson::new(),
        "algo": AlgoDatabaseKeysJson::new(),
        "db_key_prefix": DB_KEY_PREFIX.to_string(),
        "dictionary": hex::encode(EVM_ALGO_DICTIONARY_KEY.to_vec()),
        "debug_signatories": format!("0x{}", hex::encode(&*DEBUG_SIGNATORIES_DB_KEY)),
    })
    .to_string())
}
