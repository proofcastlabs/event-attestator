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
            eth_utils::get_eth_address_from_str,
            eth_crypto::eth_transaction::EthTransaction,
            eth_contracts::perc20::{
                PERC20_MIGRATE_GAS_LIMIT,
                encode_perc20_migrate_fxn_data,
            },
            eth_database_utils::{
                get_eth_chain_id_from_db,
                get_eth_gas_price_from_db,
                get_eth_private_key_from_db,
                get_eth_account_nonce_from_db,
                increment_eth_account_nonce_in_db,
                put_eth_smart_contract_address_in_db,
                get_eos_erc20_smart_contract_address_from_db,
            },
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
///     "eos_symbol": <symbol>,
///     "eth_symbol": <symbol>,
///     "eos_address": <address>,
///     "eth_address": <address>,
///     "eth_token_decimals": <num-decimals>,
///     "eos_token_decimals": <num-decimals>,
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
/// This function will remove an entry pertaining to the passed in ETH address from the
/// `EosErc20Dictionary` held in the encrypted database, should that entry exist. If it is
/// not extant, nothing is changed.
pub fn debug_remove_erc20_dictionary_entry<D>(
    db: D,
    eth_address_str: &str,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Debug removing entry from `EosErc20Dictionary`...");
    let dictionary = EosErc20Dictionary::get_from_db(&db)?;
    get_eth_address_from_str(eth_address_str)
        .and_then(|eth_address| dictionary.remove_entry_via_eth_address_and_update_in_db(&eth_address, &db))
        .and(Ok(json!({"removing_dictionary_entry_sucess":true}).to_string()))
}
/// # Debug Get PERC20 Migration Transaction
///
/// This function will create and sign a transaction that calls the `migrate` function on the
/// current `pERC20-on-EOS` smart-contract, migrationg it to the ETH address provided as an
/// argument. It then updates the smart-contract address stored in the encrypted database to that
/// new address.
///
/// ### BEWARE:
/// This function outputs a signed transaction which if NOT broadcast will result in the enclave no
/// longer working.  Use with extreme caution and only if you know exactly what you are doing!
pub fn debug_get_perc20_migration_tx<D>(
    db: D,
    new_eos_erc20_smart_contract_address_string: &str,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Debug getting migration transaction...");
    let current_eth_account_nonce = get_eth_account_nonce_from_db(&db)?;
    let current_eos_erc20_smart_contract_address = get_eos_erc20_smart_contract_address_from_db(&db)?;
    let new_eos_erc20_smart_contract_address = get_eth_address_from_str(new_eos_erc20_smart_contract_address_string)?;
    increment_eth_account_nonce_in_db(&db, 1)
        .and_then(|_| put_eth_smart_contract_address_in_db(&db, &new_eos_erc20_smart_contract_address))
        .and_then(|_| encode_perc20_migrate_fxn_data(new_eos_erc20_smart_contract_address))
        .and_then(|tx_data| Ok(EthTransaction::new_unsigned(
            tx_data,
            current_eth_account_nonce,
            0,
            current_eos_erc20_smart_contract_address,
            get_eth_chain_id_from_db(&db)?,
            PERC20_MIGRATE_GAS_LIMIT,
            get_eth_gas_price_from_db(&db)?,
        )))
        .and_then(|unsigned_tx| unsigned_tx.sign(get_eth_private_key_from_db(&db)?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .map(|hex_tx| json!({
            "success": true,
            "eth_signed_tx": hex_tx,
            "migrated_to_address:": new_eos_erc20_smart_contract_address.to_string(),
        }).to_string())
}
