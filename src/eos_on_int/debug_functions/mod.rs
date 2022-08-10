pub(crate) mod eos_block_reprocessor;
pub(crate) mod int_block_reprocessor;

use serde_json::json;

use crate::{
    chains::{eos::eos_database_utils::EosDatabaseKeysJson, eth::eth_database_utils::EthDatabaseKeysJson},
    constants::DB_KEY_PREFIX,
    debug_mode::check_debug_mode,
    dictionaries::dictionary_constants::EOS_ETH_DICTIONARY_KEY,
    types::Result,
};

/// # Debug Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    check_debug_mode().and(Ok(json!({
        "eth": EthDatabaseKeysJson::new(),
        "eos": EosDatabaseKeysJson::new(),
        "db-key-prefix": DB_KEY_PREFIX.to_string(),
        "dictionary:": hex::encode(EOS_ETH_DICTIONARY_KEY.to_vec()),
    })
    .to_string()))
}
