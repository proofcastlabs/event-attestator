use common::{
    core_type::CoreType,
    dictionaries::eos_eth::EosEthTokenDictionary,
    traits::DatabaseInterface,
    types::Result,
    utils::{convert_hex_to_eth_address, prepend_debug_output_marker_to_string},
};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
use serde_json::json;

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
        .and_then(|_| get_debug_command_hash!(function_name!(), eth_address_str, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
        .and_then(|_| convert_hex_to_eth_address(eth_address_str))
        .and_then(|eth_address| dictionary.remove_entry_via_eth_address_and_update_in_db(&eth_address, db))
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"removing_dictionary_entry_sucess":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}
