pub(crate) mod eos_block_reprocessor;
pub(crate) mod int_block_reprocessor;

use serde_json::json;

use crate::{
    chains::{
        eos::{
            core_initialization::eos_init_utils::EosInitJson,
            eos_database_utils::{EosDatabaseKeysJson, EosDbUtils},
            eos_debug_functions::{
                add_eos_eth_token_dictionary_entry,
                add_new_eos_schedule,
                remove_eos_eth_token_dictionary_entry,
                update_incremerkle,
            },
        },
        eth::{
            eth_database_utils::{EthDatabaseKeysJson, EthDbUtils},
            eth_debug_functions::debug_set_eth_gas_price_in_db as debug_set_int_gas_price_in_db,
        },
    },
    constants::{DB_KEY_PREFIX, MAX_DATA_SENSITIVITY_LEVEL},
    core_type::CoreType,
    debug_mode::{check_debug_mode, get_key_from_db, set_key_in_db_to_value},
    dictionaries::dictionary_constants::EOS_ETH_DICTIONARY_KEY,
    eos_on_int::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Update Incremerkle
///
/// This function will take an EOS initialization JSON as its input and use it to create an
/// incremerkle valid for the block number in the JSON. It will then REPLACE the incremerkle in the
/// encrypted database with this one.
///
/// ### BEWARE:
/// Changing the incremerkle changes the last block the enclave has seen and so can easily lead to
/// transaction replays. Use with extreme caution and only if you know exactly what you are doing
/// and why.
pub fn debug_update_incremerkle<D: DatabaseInterface>(
    db: &D,
    eos_init_json: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_core_is_initialized(&EthDbUtils::new(db), &EosDbUtils::new(db)).and_then(|_| {
        update_incremerkle(
            db,
            &EosInitJson::from_json_string(eos_init_json)?,
            &CoreType::EosOnInt,
            signature,
            debug_command_hash,
        )
    })
}

/// # Debug Add New Eos Schedule
///
/// Adds a new EOS schedule to the core's encrypted database.
pub fn debug_add_new_eos_schedule<D: DatabaseInterface>(
    db: &D,
    schedule_json: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_core_is_initialized(&EthDbUtils::new(db), &EosDbUtils::new(db))
        .and_then(|_| add_new_eos_schedule(db, schedule_json, &CoreType::EosOnInt, signature, debug_command_hash))
}

/// # Debug Set Key in DB to Value
///
/// This function set to the given value a given key in the encryped database.
///
/// ### BEWARE:
/// Only use this if you know exactly what you are doing and why.
pub fn debug_set_key_in_db_to_value<D: DatabaseInterface>(
    db: &D,
    key: &str,
    value: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let key_bytes = hex::decode(&key)?;
    let eos_db_utils = EosDbUtils::new(db);
    let eth_db_utils = EthDbUtils::new(db);
    let is_private_key = {
        key_bytes == eos_db_utils.get_eos_private_key_db_key() || key_bytes == eth_db_utils.get_eth_private_key_db_key()
    };
    let sensitivity = if is_private_key {
        MAX_DATA_SENSITIVITY_LEVEL
    } else {
        None
    };
    set_key_in_db_to_value(
        db,
        key,
        value,
        sensitivity,
        &CoreType::EosOnInt,
        signature,
        debug_command_hash,
    )
    .map(prepend_debug_output_marker_to_string)
}

/// # Debug Get Key From Db
///
/// This function will return the value stored under a given key in the encrypted database.
pub fn debug_get_key_from_db<D: DatabaseInterface>(
    db: &D,
    key: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let key_bytes = hex::decode(&key)?;
    let eos_db_utils = EosDbUtils::new(db);
    let eth_db_utils = EthDbUtils::new(db);
    let is_private_key = {
        key_bytes == eos_db_utils.get_eos_private_key_db_key() || key_bytes == eth_db_utils.get_eth_private_key_db_key()
    };
    let sensitivity = match is_private_key {
        true => MAX_DATA_SENSITIVITY_LEVEL,
        false => None,
    };
    get_key_from_db(db, key, sensitivity, &CoreType::EosOnInt, signature, debug_command_hash)
        .map(prepend_debug_output_marker_to_string)
}

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

/// # Debug Add Dictionary Entry
///
/// This function will add an entry to the `EosEthTokenDictionary` held in the encrypted database. The
/// dictionary defines the relationship between ERC777 interim token addresses and their pToken EOS
/// address counterparts.
///
/// The required format of an entry is:
/// {
///     "eos_symbol": <symbol>,
///     "eth_symbol": <symbol>,
///     "eos_address": <address>,
///     "eth_address": <address>,
///     "eth_token_decimals": <num-decimals>,
///     "eos_token_decimals": <num-decimals>,
/// }
pub fn debug_add_eos_eth_token_dictionary_entry<D: DatabaseInterface>(
    db: &D,
    dictionary_entry_json_string: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_core_is_initialized(&EthDbUtils::new(db), &EosDbUtils::new(db)).and_then(|_| {
        add_eos_eth_token_dictionary_entry(
            db,
            dictionary_entry_json_string,
            &CoreType::EosOnInt,
            signature,
            debug_command_hash,
        )
    })
}

/// # Debug Remove Dictionary Entry
///
/// This function will remove an entry pertaining to the passed in INT address from the
/// `EosEthTokenDictionary` held in the encrypted database, should that entry exist. If it is
/// not extant, nothing is changed.
pub fn debug_remove_eos_eth_token_dictionary_entry<D: DatabaseInterface>(
    db: &D,
    eth_address_str: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_core_is_initialized(&EthDbUtils::new(db), &EosDbUtils::new(db)).and_then(|_| {
        remove_eos_eth_token_dictionary_entry(db, eth_address_str, &CoreType::EosOnInt, signature, debug_command_hash)
    })
}

/// # Debug Set INT Gas Price
///
/// This function sets the INT gas price to use when making INT transactions. It's unit is `Wei`.
pub fn debug_set_int_gas_price<D: DatabaseInterface>(
    db: &D,
    gas_price: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    debug_set_int_gas_price_in_db(db, gas_price, &CoreType::EosOnInt, signature, debug_command_hash)
}
