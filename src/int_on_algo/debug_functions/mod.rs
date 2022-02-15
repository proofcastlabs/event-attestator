use std::str::FromStr;

use serde_json::json;

use crate::{
    chains::{
        algo::algo_database_utils::AlgoDbUtils,
        eth::{eth_database_utils::EthDbUtils, eth_utils::convert_hex_to_eth_address},
    },
    check_debug_mode::check_debug_mode,
    dictionaries::evm_algo::{EvmAlgoTokenDictionary, EvmAlgoTokenDictionaryEntry},
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
