use ethereum_types::U256;
use serde_json::json;

use crate::{
    chains::eth::{
        eth_database_utils::{EthDbUtils, EvmDbUtils},
        eth_utils::convert_hex_to_eth_address,
    },
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    dictionaries::eth_evm::EthEvmTokenDictionary,
    erc20_on_evm::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

/// # Debug Set Accrued Fees
///
/// This function updates the accrued fees value in the dictionary entry retrieved from the passed
/// in ETH address.
pub fn debug_set_accrued_fees_in_dictionary<D: DatabaseInterface>(
    db: &D,
    token_address: &str,
    fee_amount: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug setting accrued fees in dictionary...");
    let dictionary = EthEvmTokenDictionary::get_from_db(db)?;
    let dictionary_entry_eth_address = convert_hex_to_eth_address(token_address)?;
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &EvmDbUtils::new(db)))
        .and_then(|_| validate_debug_command_signature(db, &CoreType::Erc20OnEvm, signature, debug_command_hash))
        .and_then(|_| {
            dictionary.set_accrued_fees_and_save_in_db(
                db,
                &dictionary_entry_eth_address,
                U256::from_dec_str(fee_amount)?,
            )
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true,"fee":fee_amount}).to_string())
}
