use function_name::named;
pub use serde_json::json;

use crate::{
    btc_on_eos::constants::CORE_TYPE,
    chains::{
        btc::{btc_database_utils::BtcDatabaseKeysJson, utxo_manager::utxo_constants::get_utxo_constants_db_keys},
        eos::eos_database_utils::EosDatabaseKeysJson,
    },
    constants::DB_KEY_PREFIX,
    debug_functions::{validate_debug_command_signature, DEBUG_SIGNATORIES_DB_KEY},
    traits::DatabaseInterface,
    types::Result,
};

/// #  Debug Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
#[named]
pub fn debug_get_all_db_keys<D: DatabaseInterface>(db: &D, signature: &str) -> Result<String> {
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!())())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| {
            db.end_transaction()?;
            Ok(json!({
                "btc": BtcDatabaseKeysJson::new(),
                "eos": EosDatabaseKeysJson::new(),
                "db-key-prefix": DB_KEY_PREFIX.to_string(),
                "utxo-manager": get_utxo_constants_db_keys(),
                "debug_signatories": format!("0x{}", hex::encode(&*DEBUG_SIGNATORIES_DB_KEY)),
            })
            .to_string())
        })
}
