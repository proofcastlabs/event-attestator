use function_name::named;
use serde_json::json;

use crate::{
    chains::eth::{
        eth_contracts::erc20_vault::encode_erc20_vault_peg_out_fxn_data_without_user_data,
        eth_crypto::eth_transaction::EthTransaction,
        eth_database_utils::{EthDbUtils, EthDbUtilsExt, EvmDbUtils},
        eth_utils::convert_hex_to_eth_address,
    },
    debug_functions::validate_debug_command_signature,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    erc20_on_int::{check_core_is_initialized::check_core_is_initialized, constants::CORE_TYPE},
    traits::DatabaseInterface,
    types::Result,
};

/// # Debug Withdraw Fees
///
/// This function takes an address and uses it to search through the token dictionary to find a
/// corresponding entry. Once found, that entry's accrued fees are zeroed, a timestamp set in that
/// entry to mark the withdrawal date and the dictionary saved back in the database. Finally, an
/// ETH transaction is created to transfer the `<accrued_fees>` amount of tokens to the passed in
/// recipient address.
///
/// #### NOTE: This function will increment the ETH nonce and so the output transation MUST be
/// broadcast otherwise future transactions are liable to fail.
#[named]
pub fn debug_withdraw_fees_and_save_in_db<D: DatabaseInterface>(
    db: &D,
    token_address: &str,
    recipient_address: &str,
    signature: &str,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(db);
    let evm_db_utils = EvmDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &evm_db_utils))
        .and_then(|_| get_debug_command_hash!(function_name!(), token_address, recipient_address)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| EthEvmTokenDictionary::get_from_db(db))
        .and_then(|dictionary| dictionary.withdraw_fees_and_save_in_db(db, &convert_hex_to_eth_address(token_address)?))
        .and_then(|(token_address, fee_amount)| {
            let chain_id = eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                encode_erc20_vault_peg_out_fxn_data_without_user_data(
                    convert_hex_to_eth_address(recipient_address)?,
                    token_address,
                    fee_amount,
                )?,
                eth_db_utils.get_eth_account_nonce_from_db()?,
                0,
                eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                &chain_id,
                chain_id.get_erc20_vault_pegout_without_user_data_gas_limit(),
                eth_db_utils.get_eth_gas_price_from_db()?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&eth_db_utils.get_eth_private_key_from_db()?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            eth_db_utils.increment_eth_account_nonce_in_db(1)?;
            db.end_transaction()?;
            Ok(json!({"success": true, "eth_signed_tx": hex_tx}).to_string())
        })
}
