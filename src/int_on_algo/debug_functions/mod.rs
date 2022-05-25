pub(crate) mod algo_block_reprocessor;
pub(crate) mod int_block_reprocessor;

use std::str::FromStr;

use rust_algorand::{AlgorandAddress, AlgorandGenesisId, AlgorandTransaction, MicroAlgos};
use serde_json::json;

use crate::{
    chains::{
        algo::algo_database_utils::{AlgoDatabaseKeysJson, AlgoDbUtils},
        eth::{
            eth_contracts::erc20_vault::encode_erc20_vault_add_supported_token_fx_data,
            eth_crypto::eth_transaction::EthTransaction,
            eth_database_utils::{EthDatabaseKeysJson, EthDbUtils, EthDbUtilsExt},
            eth_utils::convert_hex_to_eth_address,
        },
    },
    constants::{DB_KEY_PREFIX, MAX_DATA_SENSITIVITY_LEVEL},
    debug_mode::{check_debug_mode, get_key_from_db, set_key_in_db_to_value},
    dictionaries::{
        dictionary_constants::EVM_ALGO_DICTIONARY_KEY,
        evm_algo::{EvmAlgoTokenDictionary, EvmAlgoTokenDictionaryEntry},
    },
    int_on_algo::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
    utils::strip_hex_prefix,
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

/// # Debug Get Add Supported Token Transaction
///
/// This function will sign a transaction to add the given address as a supported token to
/// the `erc20-vault` smart-contract.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// This function will increment the core's ETH nonce, and so if the transaction is not broadcast
/// successfully, the core's ETH side will no longer function correctly. Use with extreme caution
/// and only if you know exactly what you are doing and why!
pub fn debug_get_add_supported_token_tx<D: DatabaseInterface>(db: &D, eth_address_str: &str) -> Result<String> {
    info!("✔ Debug getting `addSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDbUtils::new(db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = convert_hex_to_eth_address(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &AlgoDbUtils::new(db)))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_add_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            let chain_id = eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_int_on_algo_smart_contract_address()?,
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

/// # Debug Get Algo Pay Tx
///
/// This function will create an Algorand `pay` tx type using the passed in arguments and signed
/// by the algorand key saved in the encrypted database.
///
/// __NOTE:__ This function will _not_ increment the ALGO signature nonce!
pub fn debug_get_algo_pay_tx<D: DatabaseInterface>(
    db: &D,
    first_valid: u64,
    genesis_id: &str,
    fee: u64,
    receiver: &str,
    note: &str,
    amount: u64,
) -> Result<String> {
    info!("✔ Getting ALGO pay tx...");
    let algo_db_utils = AlgoDbUtils::new(db);
    // TODO If the note is valid hex, use it raw, else if is valid utf8, convert it to bytes.
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &algo_db_utils))
        .and_then(|_| {
            let pk = algo_db_utils.get_algo_private_key()?;
            let note_bytes = hex::decode(strip_hex_prefix(note))?;
            Ok(AlgorandTransaction::new_payment_tx(
                amount,
                MicroAlgos::new(fee),
                if note_bytes.is_empty() { None } else { Some(note_bytes) },
                first_valid,
                pk.to_address()?,
                AlgorandAddress::from_str(receiver)?,
                AlgorandGenesisId::from_str(genesis_id)?.hash()?,
                None,
            )?
            .sign(&pk)?
            .to_hex()?)
        })
}
