use common::{constants::DB_KEY_PREFIX, traits::DatabaseInterface, types::Result};
use common_btc::{get_utxo_constants_db_keys, BtcDatabaseKeysJson, BTC_ON_ETH_FEE_DB_KEYS};
use common_debug_signers::{validate_debug_command_signature, DEBUG_SIGNATORIES_DB_KEY};
use common_eth::EthDatabaseKeysJson;
use function_name::named;
use serde_json::json;

use crate::constants::CORE_TYPE;

/// # Debug Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
#[named]
pub fn debug_get_all_db_keys<D: DatabaseInterface>(db: &D, signature: &str) -> Result<String> {
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!())())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| {
            db.end_transaction()?;
            Ok(json!({
                "btc": BtcDatabaseKeysJson::new(),
                "eth": EthDatabaseKeysJson::new(),
                "fees": BTC_ON_ETH_FEE_DB_KEYS.to_json(),
                "db-key-prefix": DB_KEY_PREFIX.to_string(),
                "utxo-manager": get_utxo_constants_db_keys(),
                "debug_signatories": format!("0x{}", hex::encode(*DEBUG_SIGNATORIES_DB_KEY)),
            })
            .to_string())
        })
}
