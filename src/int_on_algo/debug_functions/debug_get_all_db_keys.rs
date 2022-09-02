use function_name::named;
use serde_json::json;

use crate::{
    chains::{algo::algo_database_utils::AlgoDatabaseKeysJson, eth::eth_database_utils::EthDatabaseKeysJson},
    constants::DB_KEY_PREFIX,
    debug_functions::{validate_debug_command_signature, DEBUG_SIGNATORIES_DB_KEY},
    dictionaries::dictionary_constants::EVM_ALGO_DICTIONARY_KEY,
    int_on_algo::constants::CORE_TYPE,
    traits::DatabaseInterface,
    types::Result,
};

/// # Debug Get All DB Keys
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
                "int": EthDatabaseKeysJson::new(),
                "algo": AlgoDatabaseKeysJson::new(),
                "db_key_prefix": DB_KEY_PREFIX.to_string(),
                "dictionary": hex::encode(EVM_ALGO_DICTIONARY_KEY.to_vec()),
                "debug_signatories": format!("0x{}", hex::encode(&*DEBUG_SIGNATORIES_DB_KEY)),
            })
            .to_string())
        })
}
