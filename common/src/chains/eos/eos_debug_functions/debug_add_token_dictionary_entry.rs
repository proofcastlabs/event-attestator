use std::str::FromStr;

use function_name::named;
use serde_json::json;

use crate::{
    core_type::CoreType,
    debug_functions::validate_debug_command_signature,
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryEntry},
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Add ERC20 Token Dictionary Entry
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
#[named]
pub fn debug_add_token_dictionary_entry<D: DatabaseInterface>(
    db: &D,
    dictionary_entry_json_string: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug adding entry to `EosEthTokenDictionary`...");
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), dictionary_entry_json_string, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| {
            EosEthTokenDictionary::get_from_db(db)?
                .add_and_update_in_db(EosEthTokenDictionaryEntry::from_str(dictionary_entry_json_string)?, db)
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"adding_dictionary_entry_sucess":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}
