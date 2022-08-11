pub use serde_json::json;

use crate::{
    chains::{
        btc::{btc_database_utils::BtcDatabaseKeysJson, utxo_manager::utxo_constants::get_utxo_constants_db_keys},
        eos::eos_database_utils::EosDatabaseKeysJson,
    },
    constants::DB_KEY_PREFIX,
    types::Result,
};

/// #  Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn get_all_db_keys() -> Result<String> {
    Ok(json!({
        "btc": BtcDatabaseKeysJson::new(),
        "eos": EosDatabaseKeysJson::new(),
        "db-key-prefix": DB_KEY_PREFIX.to_string(),
        "utxo-manager": get_utxo_constants_db_keys(),
    })
    .to_string())
}
