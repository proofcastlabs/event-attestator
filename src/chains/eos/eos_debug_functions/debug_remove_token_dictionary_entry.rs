use function_name::named;
use serde_json::json;

use crate::{
    chains::eth::eth_utils::get_eth_address_from_str,
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    dictionaries::eos_eth::EosEthTokenDictionary,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Remove ERC20 Token Dictionary Entry
///
/// This function will remove an entry pertaining to the passed in ETH address from the
/// `EosEthTokenDictionary` held in the encrypted database, should that entry exist. If it is
/// not extant, nothing is changed.
#[named]
pub fn debug_remove_token_dictionary_entry<D: DatabaseInterface>(
    db: &D,
    eth_address_str: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug removing entry from `EosEthTokenDictionary`...");
    let dictionary = EosEthTokenDictionary::get_from_db(db)?;
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), eth_address_str, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| get_eth_address_from_str(eth_address_str))
        .and_then(|eth_address| dictionary.remove_entry_via_eth_address_and_update_in_db(&eth_address, db))
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"removing_dictionary_entry_sucess":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}
