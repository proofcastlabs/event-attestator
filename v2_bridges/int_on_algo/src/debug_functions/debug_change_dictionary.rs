use std::str::FromStr;

use common::{
    core_type::CoreType,
    dictionaries::evm_algo::{EvmAlgoTokenDictionary, EvmAlgoTokenDictionaryEntry},
    traits::DatabaseInterface,
    types::Result,
};
use common_debug_signers::validate_debug_command_signature;
use common_eth::convert_hex_to_eth_address;
use function_name::named;
use serde_json::json;

use crate::constants::CORE_TYPE;

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
#[named]
pub fn debug_add_dictionary_entry<D: DatabaseInterface>(db: &D, json_str: &str, signature: &str) -> Result<String> {
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), json_str)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
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
#[named]
pub fn debug_remove_dictionary_entry<D: DatabaseInterface>(
    db: &D,
    eth_address_str: &str,
    signature: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), eth_address_str)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| EvmAlgoTokenDictionary::get_from_db(db))
        .and_then(|dictionary| {
            dictionary.remove_entry_via_evm_address_and_update_in_db(&convert_hex_to_eth_address(eth_address_str)?, db)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"remove_dictionary_entry_success:":"true"}).to_string())
}
