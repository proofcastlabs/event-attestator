use common::{
    chains::eth::eth_utils::convert_hex_to_eth_address,
    core_type::CoreType,
    debug_functions::validate_debug_command_signature,
    dictionaries::eos_eth::EosEthTokenDictionary,
    traits::DatabaseInterface,
    types::Result,
};
use ethereum_types::U256;
use function_name::named;
use serde_json::json;

use crate::constants::CORE_TYPE;

/// # Debug Set Accrued Fees
///
/// This function updates the accrued fees value in the dictionary entry retrieved from the passed
/// in ETH address.
#[named]
pub fn debug_set_accrued_fees_in_dictionary<D: DatabaseInterface>(
    db: &D,
    token_address: &str,
    fee_amount: &str,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug setting accrued fees in dictionary...");
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), token_address, &fee_amount)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| {
            EosEthTokenDictionary::get_from_db(db)?.set_accrued_fees_and_save_in_db(
                db,
                &convert_hex_to_eth_address(token_address)?,
                U256::from_dec_str(fee_amount)?,
            )
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true,"fee":fee_amount}).to_string())
}
