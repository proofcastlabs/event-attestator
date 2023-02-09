use common::{
    chains::eth::eth_utils::convert_hex_to_eth_address,
    core_type::CoreType,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    fees::fee_utils::sanity_check_basis_points_value,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
use serde_json::json;

use crate::constants::CORE_TYPE;

/// # Debug Set Fee Basis Points
///
/// This function takes an address and a new fee param. It gets the `EthEvmTokenDictionary` from
/// the database then finds the entry pertaining to the address in question and if successful,
/// updates the fee associated with that address before saving the dictionary back into the
/// database. If no entry is found for a given `address` the function will return an error saying
/// as such.
///
/// #### NOTE: Using a fee of 0 will mean no fees are taken.
#[named]
pub fn debug_set_fee_basis_points<D: DatabaseInterface>(
    db: &D,
    address: &str,
    new_fee: u64,
    signature: &str,
) -> Result<String> {
    db.start_transaction()
        .map(|_| sanity_check_basis_points_value(new_fee))
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), address, &new_fee)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| EthEvmTokenDictionary::get_from_db(db))
        .and_then(|dictionary| {
            dictionary.change_fee_basis_points_and_update_in_db(db, &convert_hex_to_eth_address(address)?, new_fee)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true, "address": address, "new_fee": new_fee}).to_string())
        .map(prepend_debug_output_marker_to_string)
}
