use serde_json::json;

use crate::{
    chains::eth::{
        eth_database_utils::{EthDbUtils, EvmDbUtils},
        eth_utils::convert_hex_to_eth_address,
    },
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    dictionaries::eth_evm::{EthEvmTokenDictionary, EthEvmTokenDictionaryEntry},
    erc20_on_evm::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

/// # Debug Add Dictionary Entry
///
/// This function will add an entry to the `EthEvmTokenDictionary` held in the encrypted database. The
/// dictionary defines the relationship between ETH token addresses and the address of their pTokenized,
/// EVM-compliant counterparts.
///
/// The required format of an entry is:
/// {
///     "eth_symbol": <symbol>,
///     "evm_symbol": <symbol>,
///     "eth_address": <address>,
///     "evm_address": <address>,
/// }
pub fn debug_add_dictionary_entry<D: DatabaseInterface>(
    db: &D,
    json_str: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &EvmDbUtils::new(db)))
        .and_then(|_| validate_debug_command_signature(db, &CoreType::Erc20OnEvm, signature, debug_command_hash))
        .and_then(|_| EthEvmTokenDictionary::get_from_db(db))
        .and_then(|dictionary| dictionary.add_and_update_in_db(EthEvmTokenDictionaryEntry::from_str(json_str)?, db))
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"add_dictionary_entry_success:":"true"}).to_string())
}

/// # Debug Remove Dictionary Entry
///
/// This function will remove an entry pertaining to the passed in ETH address from the
/// `EthEvmTokenDictionaryEntry` held in the encrypted database, should that entry exist. If it is
/// not extant, nothing is changed.
pub fn debug_remove_dictionary_entry<D: DatabaseInterface>(
    db: &D,
    eth_address_str: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &EvmDbUtils::new(db)))
        .and_then(|_| validate_debug_command_signature(db, &CoreType::Erc20OnEvm, signature, debug_command_hash))
        .and_then(|_| EthEvmTokenDictionary::get_from_db(db))
        .and_then(|dictionary| {
            dictionary.remove_entry_via_eth_address_and_update_in_db(&convert_hex_to_eth_address(eth_address_str)?, db)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"remove_dictionary_entry_success:":"true"}).to_string())
}
