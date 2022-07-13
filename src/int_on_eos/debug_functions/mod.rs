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
                get_processed_actions_list,
                remove_eos_eth_token_dictionary_entry,
                update_incremerkle,
            },
        },
        eth::{
            eth_contracts::erc20_vault::{
                encode_erc20_vault_add_supported_token_fx_data,
                encode_erc20_vault_remove_supported_token_fx_data,
            },
            eth_crypto::eth_transaction::EthTransaction,
            eth_database_utils::{EthDatabaseKeysJson, EthDbUtils, EthDbUtilsExt},
            eth_debug_functions::debug_set_eth_gas_price_in_db,
            eth_utils::get_eth_address_from_str,
        },
    },
    check_debug_mode::check_debug_mode,
    constants::{DB_KEY_PREFIX, MAX_DATA_SENSITIVITY_LEVEL},
    debug_database_utils::{get_key_from_db, set_key_in_db_to_value},
    dictionaries::dictionary_constants::EOS_ETH_DICTIONARY_KEY,
    int_on_eos::check_core_is_initialized::check_core_is_initialized,
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
pub fn debug_update_incremerkle<D: DatabaseInterface>(db: &D, eos_init_json: &str) -> Result<String> {
    check_core_is_initialized(&EthDbUtils::new(db), &EosDbUtils::new(db))
        .and_then(|_| update_incremerkle(db, &EosInitJson::from_json_string(eos_init_json)?))
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Add New Eos Schedule
///
/// Adds a new EOS schedule to the core's encrypted database.
pub fn debug_add_new_eos_schedule<D: DatabaseInterface>(db: D, schedule_json: &str) -> Result<String> {
    check_core_is_initialized(&EthDbUtils::new(&db), &EosDbUtils::new(&db))
        .and_then(|_| add_new_eos_schedule(&db, schedule_json))
}

/// # Debug Set Key in DB to Value
///
/// This function set to the given value a given key in the encryped database.
///
/// ### BEWARE:
/// Only use this if you know exactly what you are doing and why.
pub fn debug_set_key_in_db_to_value<D: DatabaseInterface>(db: D, key: &str, value: &str) -> Result<String> {
    let key_bytes = hex::decode(&key)?;
    let eos_db_utils = EosDbUtils::new(&db);
    let eth_db_utils = EthDbUtils::new(&db);
    let is_private_key = {
        key_bytes == eos_db_utils.get_eos_private_key_db_key() || key_bytes == eth_db_utils.get_eth_private_key_db_key()
    };
    let sensitivity = if is_private_key {
        MAX_DATA_SENSITIVITY_LEVEL
    } else {
        None
    };
    set_key_in_db_to_value(db, key, value, sensitivity).map(prepend_debug_output_marker_to_string)
}

/// # Debug Get Key From Db
///
/// This function will return the value stored under a given key in the encrypted database.
pub fn debug_get_key_from_db<D: DatabaseInterface>(db: D, key: &str) -> Result<String> {
    let key_bytes = hex::decode(&key)?;
    let eos_db_utils = EosDbUtils::new(&db);
    let eth_db_utils = EthDbUtils::new(&db);
    let is_private_key = {
        key_bytes == eos_db_utils.get_eos_private_key_db_key() || key_bytes == eth_db_utils.get_eth_private_key_db_key()
    };
    let sensitivity = if is_private_key {
        MAX_DATA_SENSITIVITY_LEVEL
    } else {
        None
    };
    get_key_from_db(db, key, sensitivity).map(prepend_debug_output_marker_to_string)
}

/// # Debug Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    check_debug_mode().and(Ok(json!({
        "eth": EthDatabaseKeysJson::new(),
        "eos": EosDatabaseKeysJson::new(),
        "db-key-prefix": DB_KEY_PREFIX.to_string(),
        "dictionary": hex::encode(EOS_ETH_DICTIONARY_KEY.to_vec()),
    })
    .to_string()))
}

/// # Debug Add ERC20 Dictionary Entry
///
/// This function will add an entry to the `EosEthTokenDictionary` held in the encrypted database. The
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
pub fn debug_add_token_dictionary_entry<D: DatabaseInterface>(
    db: D,
    dictionary_entry_json_string: &str,
) -> Result<String> {
    check_core_is_initialized(&EthDbUtils::new(&db), &EosDbUtils::new(&db))
        .and_then(|_| add_eos_eth_token_dictionary_entry(&db, dictionary_entry_json_string))
}

/// # Debug Remove ERC20 Dictionary Entry
///
/// This function will remove an entry pertaining to the passed in ETH address from the
/// `EosEthTokenDictionary` held in the encrypted database, should that entry exist. If it is
/// not extant, nothing is changed.
pub fn debug_remove_token_dictionary_entry<D: DatabaseInterface>(db: D, eth_address_str: &str) -> Result<String> {
    check_core_is_initialized(&EthDbUtils::new(&db), &EosDbUtils::new(&db))
        .and_then(|_| remove_eos_eth_token_dictionary_entry(&db, eth_address_str))
}

/// # Debug Get Add Supported Token Transaction
///
/// This function will sign a transaction to add the given address as a supported token to
/// the `erc20-vault-on-eos` smart-contract.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// This function will increment the core's ETH nonce, and so if the transaction is not broadcast
/// successfully, the core's ETH side will no longer function correctly. Use with extreme caution
/// and only if you know exactly what you are doing and why!
pub fn debug_get_add_supported_token_tx<D: DatabaseInterface>(db: D, eth_address_str: &str) -> Result<String> {
    info!("✔ Debug getting `addSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDbUtils::new(&db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = get_eth_address_from_str(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &EosDbUtils::new(&db)))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_add_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            let chain_id = &eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_int_on_eos_smart_contract_address_from_db()?,
                chain_id,
                chain_id.get_erc20_vault_change_supported_token_gas_limit(),
                eth_db_utils.get_eth_gas_price_from_db()?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&eth_db_utils.get_eth_private_key_from_db()?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            db.end_transaction()?;
            Ok(json!({ "success": true, "eth_signed_tx": hex_tx }).to_string())
        })
}

/// # Debug Get Remove Supported Token Transaction
///
/// This function will sign a transaction to remove the given address as a supported token to
/// the `erc20-vault-on-eos` smart-contract.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// This function will increment the core's ETH nonce, and so if the transaction is not broadcast
/// successfully, the core's ETH side will no longer function correctly. Use with extreme caution
/// and only if you know exactly what you are doing and why!
pub fn debug_get_remove_supported_token_tx<D: DatabaseInterface>(db: D, eth_address_str: &str) -> Result<String> {
    info!("✔ Debug getting `removeSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDbUtils::new(&db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = get_eth_address_from_str(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &EosDbUtils::new(&db)))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_remove_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            let chain_id = eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_int_on_eos_smart_contract_address_from_db()?,
                &chain_id,
                chain_id.get_erc20_vault_change_supported_token_gas_limit(),
                eth_db_utils.get_eth_gas_price_from_db()?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&eth_db_utils.get_eth_private_key_from_db()?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            db.end_transaction()?;
            Ok(json!({ "success": true, "eth_signed_tx": hex_tx }).to_string())
        })
}

/// # Debug Get Processed Actions List
///
/// This function returns the list of already-processed action global sequences in JSON format.
pub fn debug_get_processed_actions_list<D: DatabaseInterface>(db: &D) -> Result<String> {
    check_core_is_initialized(&EthDbUtils::new(db), &EosDbUtils::new(db)).and_then(|_| get_processed_actions_list(db))
}

/// # Debug Set ETH Gas Price
///
/// This function sets the ETH gas price to use when making ETH transactions. It's unit is `Wei`.
pub fn debug_set_int_gas_price<D: DatabaseInterface>(db: D, gas_price: u64) -> Result<String> {
    debug_set_eth_gas_price_in_db(&db, gas_price)
}
