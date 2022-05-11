pub(crate) mod block_reprocessors;

use std::str::FromStr;

use serde_json::json;

use crate::{
    chains::{
        algo::algo_database_utils::{AlgoDatabaseKeysJson, AlgoDbUtils},
        eth::{
            eth_database_utils::{EthDatabaseKeysJson, EthDbUtils},
            eth_utils::convert_hex_to_eth_address,
        },
    },
    check_debug_mode::check_debug_mode,
    constants::{DB_KEY_PREFIX, MAX_DATA_SENSITIVITY_LEVEL},
    debug_database_utils::{get_key_from_db, set_key_in_db_to_value},
    dictionaries::{
        dictionary_constants::EVM_ALGO_DICTIONARY_KEY,
        evm_algo::{EvmAlgoTokenDictionary, EvmAlgoTokenDictionaryEntry},
    },
    int_on_algo::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

/// # Debug Add Dictionary Entry
///
/// This function will add an entry to the `EvmAlgoTokenDictionary` held in the encrypted database. The
/// dictionary defines the relationship between EVM token addresses and the asset ID of their pTokenized,
/// ALGO-compliant counterparts.
///
/// The required format of an entry is:
/// {
///     "eth_symbol": <symbol>,
///     "evm_symbol": <symbol>,
///     "eth_address": <address>,
///     "evm_address": <address>,
/// }
pub fn debug_add_dictionary_entry<D: DatabaseInterface>(db: &D, json_str: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &AlgoDbUtils::new(db)))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EvmAlgoTokenDictionary::get_from_db(db))
        .and_then(|dictionary| dictionary.add_and_update_in_db(EvmAlgoTokenDictionaryEntry::from_str(json_str)?, db))
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"add_dictionary_entry_success:":"true"}).to_string())
}

/// # Debug Remove Dictionary Entry
///
/// This function will remove an entry pertaining to the passed in EVM address from the
/// `EvmAlgoTokenDictionaryEntry` held in the encrypted database, should that entry exist. If it is
/// not extant, nothing is changed.
pub fn debug_remove_dictionary_entry<D: DatabaseInterface>(db: &D, eth_address_str: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &AlgoDbUtils::new(db)))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EvmAlgoTokenDictionary::get_from_db(db))
        .and_then(|dictionary| {
            dictionary.remove_entry_via_evm_address_and_update_in_db(&convert_hex_to_eth_address(eth_address_str)?, db)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"remove_dictionary_entry_success:":"true"}).to_string())
}

/// Debug Set Algo Account Nonce
///
/// Sets the Algo account nonce in the database to the passed in value.
pub fn debug_set_algo_account_nonce<D: DatabaseInterface>(db: &D, nonce: u64) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &AlgoDbUtils::new(db)))
        .and_then(|_| db.start_transaction())
        .and_then(|_| AlgoDbUtils::new(db).put_algo_account_nonce_in_db(nonce))
        .and_then(|_| db.end_transaction())
        .map(|_| json!({ "algo_account_nonce": nonce }).to_string())
}

/// # Debug Get All DB Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    check_debug_mode().map(|_| {
        json!({
            "int": EthDatabaseKeysJson::new(),
            "algo": AlgoDatabaseKeysJson::new(),
            "db-key-prefix": DB_KEY_PREFIX.to_string(),
            "dictionary": hex::encode(EVM_ALGO_DICTIONARY_KEY.to_vec()),
        })
        .to_string()
    })
}

/// # Debug Set Key in DB to Value
///
/// This function set to the given value a given key in the encryped database.
///
/// ### BEWARE:
/// Only use this if you know exactly what you are doing and why.
pub fn debug_set_key_in_db_to_value<D: DatabaseInterface>(db: D, key: &str, value: &str) -> Result<String> {
    check_debug_mode().and_then(|_| {
        let key_bytes = hex::decode(&key)?;
        let sensitivity = if key_bytes == EthDbUtils::new(&db).get_eth_private_key_db_key()
            || key_bytes == AlgoDbUtils::new(&db).get_algo_private_key_key()
        {
            MAX_DATA_SENSITIVITY_LEVEL
        } else {
            None
        };
        set_key_in_db_to_value(db, key, value, sensitivity)
    })
}

/// # Debug Get Key From Db
///
/// This function will return the value stored under a given key in the encrypted database.
pub fn debug_get_key_from_db<D: DatabaseInterface>(db: D, key: &str) -> Result<String> {
    check_debug_mode().and_then(|_| {
        let key_bytes = hex::decode(&key)?;
        let sensitivity = if key_bytes == EthDbUtils::new(&db).get_eth_private_key_db_key()
            || key_bytes == AlgoDbUtils::new(&db).get_algo_private_key_key()
        {
            MAX_DATA_SENSITIVITY_LEVEL
        } else {
            None
        };
        get_key_from_db(db, key, sensitivity)
    })
}
