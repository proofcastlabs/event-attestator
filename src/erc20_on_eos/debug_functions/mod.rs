pub use serde_json::json;
use crate::{
    types::Result,
    traits::DatabaseInterface,
    check_debug_mode::check_debug_mode,
    utils::prepend_debug_output_marker_to_string,
    constants::{
        DB_KEY_PREFIX,
        PRIVATE_KEY_DATA_SENSITIVITY_LEVEL,
    },
    debug_database_utils::{
        get_key_from_db,
        set_key_in_db_to_value,
    },
    erc20_on_eos::check_core_is_initialized::check_core_is_initialized,
    chains::{
        eos::{
            eos_database_utils::put_eos_schedule_in_db,
            parse_eos_schedule::parse_v2_schedule_string_to_v2_schedule,
            eos_erc20_dictionary::{
                EosErc20Dictionary,
                EosErc20DictionaryEntry,
            },
            eos_constants::{
                EOS_PRIVATE_KEY_DB_KEY,
                get_eos_constants_db_keys,
            },
            core_initialization::eos_init_utils::{
                EosInitJson,
                put_eos_latest_block_info_in_db,
                generate_and_put_incremerkle_in_db,
            },
        },
        eth::{
            eth_constants::{
                ETH_PRIVATE_KEY_DB_KEY,
                get_eth_constants_db_keys,
            },
        },
    },
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
pub fn debug_update_incremerkle<D: DatabaseInterface>(db: &D, eos_init_json: &str) -> Result<String> {
    info!("✔ Debug updating blockroot merkle...");
    let init_json = EosInitJson::from_json_string(&eos_init_json)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(db))
        .and_then(|_| put_eos_latest_block_info_in_db(db, &init_json.block))
        .and_then(|_| db.start_transaction())
        .and_then(|_| generate_and_put_incremerkle_in_db(db, &init_json.blockroot_merkle))
        .and_then(|_| db.end_transaction())
        .and(Ok("{debug_update_blockroot_merkle_success:true}".to_string()))
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Add New Eos Schedule
///
/// Does exactly what it says on the tin. It's currently required due to an open ticket on the
/// validation of EOS blocks containing new schedules. Once that ticket is cleared, new schedules
/// can be brought in "organically" by syncing to the core up to the block containing said new
/// schedule. Meanwhile, this function must suffice.
pub fn debug_add_new_eos_schedule<D: DatabaseInterface>(db: D, schedule_json: &str) -> Result<String> {
    info!("✔ Debug adding new EOS schedule...");
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| parse_v2_schedule_string_to_v2_schedule(&schedule_json))
        .and_then(|schedule| put_eos_schedule_in_db(&db, &schedule))
        .and_then(|_| db.end_transaction())
        .and(Ok("{debug_adding_eos_schedule_succeeded:true}".to_string()))
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Set Key in DB to Value
///
/// This function set to the given value a given key in the encryped database.
///
/// ### BEWARE:
/// Only use this if you know exactly what you are doing and why.
pub fn debug_set_key_in_db_to_value<D: DatabaseInterface>(db: D, key: &str, value: &str) -> Result<String> {
    let key_bytes = hex::decode(&key)?;
    let is_private_key = {
        key_bytes == EOS_PRIVATE_KEY_DB_KEY.to_vec() || key_bytes == ETH_PRIVATE_KEY_DB_KEY.to_vec()
    };
    let sensitivity = match is_private_key {
        true => PRIVATE_KEY_DATA_SENSITIVITY_LEVEL,
        false => None,
    };
    set_key_in_db_to_value(db, key, value, sensitivity).map(prepend_debug_output_marker_to_string)
}

/// # Debug Get Key From Db
///
/// This function will return the value stored under a given key in the encrypted database.
pub fn debug_get_key_from_db<D: DatabaseInterface>(db: D, key: &str) -> Result<String> {
    let key_bytes = hex::decode(&key)?;
    let is_private_key = {
        key_bytes == EOS_PRIVATE_KEY_DB_KEY.to_vec() || key_bytes == ETH_PRIVATE_KEY_DB_KEY.to_vec()
    };
    let sensitivity = match is_private_key {
        true => PRIVATE_KEY_DATA_SENSITIVITY_LEVEL,
        false => None,
    };
    get_key_from_db(db, key, sensitivity).map(prepend_debug_output_marker_to_string)
}

/// # Debug Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    check_debug_mode()
        .and(Ok(json!({
            "eth": get_eth_constants_db_keys(),
            "eos": get_eos_constants_db_keys(),
            "db-key-prefix": DB_KEY_PREFIX.to_string(),
        }).to_string())
    )
}

/// # Debug Add ERC20 Dictionary Entry
///
/// This function will add an entry to the `EosErc20Dictionary` held in the encrypted database. The
/// dictionary defines the relationship between ERC20 etheruem addresses and their pToken EOS
/// address counterparts.
///
/// The required format of an entry is:
/// {
///     "eos_token_account_name":"<eos-account-name>",
///     "eth_erc20_token_address":"<erc20-token-address>"
/// }
pub fn debug_add_erc20_dictionary_entry<D>(
    db: D,
    dictionary_entry_json_string: &str,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Debug adding entry to `EosErc20Dictionary`...");
    let dictionary = EosErc20Dictionary::get_from_db(&db)?;
    EosErc20DictionaryEntry::from_str(dictionary_entry_json_string)
        .and_then(|entry| dictionary.add_and_update_in_db(entry, &db))
        .and(Ok(json!({"adding_dictionary_entry_sucess":true}).to_string()))
}

/// # Debug Remove ERC20 Dictionary Entry
///
/// This function will remove an entry to the `EosErc20Dictionary` held in the encrypted database. The
/// dictionary defines the relationship between ERC20 etheruem addresses and their pToken EOS
/// address counterparts.
///
/// The required format of an entry is:
/// {
///     "eos_token_account_name":"<eos-account-name>",
///     "eth_erc20_token_address":"<erc20-token-address>"
/// }
pub fn debug_remove_erc20_dictionary_entry<D>(
    db: D,
    dictionary_entry_json_string: &str,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Debug adding entry to `EosErc20Dictionary`...");
    let dictionary = EosErc20Dictionary::get_from_db(&db)?;
    EosErc20DictionaryEntry::from_str(dictionary_entry_json_string)
        .and_then(|entry| dictionary.remove_and_update_in_db(&entry, &db))
        .and(Ok(json!({"removing_dictionary_entry_sucess":true}).to_string()))
}
