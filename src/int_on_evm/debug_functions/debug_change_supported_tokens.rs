use serde_json::json;

use crate::{
    chains::eth::{
        eth_contracts::erc20_vault::{
            encode_erc20_vault_add_supported_token_fx_data,
            encode_erc20_vault_remove_supported_token_fx_data,
        },
        eth_crypto::eth_transaction::EthTransaction,
        eth_database_utils::{EthDbUtils, EthDbUtilsExt, EvmDbUtils},
        eth_utils::convert_hex_to_eth_address,
    },
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    int_on_evm::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

/// # Debug Get Add Supported Token Transaction
///
/// This function will sign a transaction to add the given address as a supported token to
/// the `erc20-vault-on-eos` smart-contract.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// This function will increment the core's ETH nonce, and so if the transaction is not broadcast
/// successfully, the core's ETH side will no longer function correctly. Use with extreme caution
/// and only if you know exactly what you are doing and why!
pub fn debug_get_add_supported_token_tx<D: DatabaseInterface>(
    db: &D,
    eth_address_str: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug getting `addSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDbUtils::new(db);
    let evm_db_utils = EvmDbUtils::new(db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = convert_hex_to_eth_address(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &evm_db_utils))
        .and_then(|_| validate_debug_command_signature(db, &CoreType::IntOnEvm, signature, debug_command_hash))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_add_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            let chain_id = eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                &chain_id,
                chain_id.get_erc20_vault_change_supported_token_gas_limit(),
                eth_db_utils.get_eth_gas_price_from_db()?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&eth_db_utils.get_eth_private_key_from_db()?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            db.end_transaction()?;
            Ok(json!({ "success": true, "eth_signed_tx": hex_tx }).to_string())
        })
}

/// # Debug Get Remove Supported Token Transaction
///
/// This function will sign a transaction to remove the given address as a supported token to
/// the `erc20-vault-on-eos` smart-contract.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// This function will increment the core's ETH nonce, and so if the transaction is not broadcast
/// successfully, the core's ETH side will no longer function correctly. Use with extreme caution
/// and only if you know exactly what you are doing and why!
pub fn debug_get_remove_supported_token_tx<D: DatabaseInterface>(
    db: &D,
    eth_address_str: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug getting `removeSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDbUtils::new(db);
    let evm_db_utils = EvmDbUtils::new(db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = convert_hex_to_eth_address(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &evm_db_utils))
        .and_then(|_| validate_debug_command_signature(db, &CoreType::IntOnEvm, signature, debug_command_hash))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_remove_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            let chain_id = eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                &chain_id,
                chain_id.get_erc20_vault_change_supported_token_gas_limit(),
                eth_db_utils.get_eth_gas_price_from_db()?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&eth_db_utils.get_eth_private_key_from_db()?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            db.end_transaction()?;
            Ok(json!({ "success": true, "eth_signed_tx": hex_tx }).to_string())
        })
}
