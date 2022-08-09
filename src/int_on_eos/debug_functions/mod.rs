pub(crate) mod eos_block_reprocessor;
pub(crate) mod int_block_reprocessor;

use serde_json::json;

use crate::{
    chains::{
        eos::eos_database_utils::{EosDatabaseKeysJson, EosDbUtils},
        eth::{
            eth_contracts::erc20_vault::{
                encode_erc20_vault_add_supported_token_fx_data,
                encode_erc20_vault_remove_supported_token_fx_data,
            },
            eth_crypto::eth_transaction::EthTransaction,
            eth_database_utils::{EthDatabaseKeysJson, EthDbUtils, EthDbUtilsExt},
            eth_utils::get_eth_address_from_str,
        },
    },
    constants::{DB_KEY_PREFIX, MAX_DATA_SENSITIVITY_LEVEL},
    core_type::CoreType,
    debug_mode::{
        check_debug_mode,
        get_key_from_db,
        set_key_in_db_to_value,
        validate_debug_command_signature,
        DEBUG_SIGNATORIES_DB_KEY,
    },
    dictionaries::dictionary_constants::EOS_ETH_DICTIONARY_KEY,
    int_on_eos::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

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
        &CoreType::IntOnEos,
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
    let sensitivity = if is_private_key {
        MAX_DATA_SENSITIVITY_LEVEL
    } else {
        None
    };
    get_key_from_db(db, key, sensitivity, &CoreType::IntOnEos, signature, debug_command_hash)
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    check_debug_mode().and(Ok(json!({
        "eth": EthDatabaseKeysJson::new(),
        "eos": EosDatabaseKeysJson::new(),
        "db_key_prefix": DB_KEY_PREFIX.to_string(),
        "dictionary": hex::encode(EOS_ETH_DICTIONARY_KEY.to_vec()),
        "debug_signatories": format!("0x{}", hex::encode(&*DEBUG_SIGNATORIES_DB_KEY)),
    })
    .to_string()))
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
pub fn debug_get_add_supported_token_tx<D: DatabaseInterface>(
    db: &D,
    eth_address_str: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug getting `addSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDbUtils::new(db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = get_eth_address_from_str(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &EosDbUtils::new(db)))
        .and_then(|_| validate_debug_command_signature(db, &CoreType::IntOnEos, signature, debug_command_hash))
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
pub fn debug_get_remove_supported_token_tx<D: DatabaseInterface>(
    db: &D,
    eth_address_str: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug getting `removeSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDbUtils::new(db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = get_eth_address_from_str(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &EosDbUtils::new(db)))
        .and_then(|_| validate_debug_command_signature(db, &CoreType::IntOnEos, signature, debug_command_hash))
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
