
use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_debug_signers::validate_debug_command_signature;
use common_eth::{
    convert_hex_to_eth_address,
    encode_erc20_vault_migrate_single_fxn_data,
    EthDbUtils,
    EthDbUtilsExt,
    EthTransaction,
};
use function_name::named;
use serde_json::json;

use crate::constants::CORE_TYPE;

/// # Debug Get EthOnEvmVault Single Migration Transaction
///
/// This function will create and sign a transaction that calls the `migrateSingle` function on
/// the `pERC20-on-EVM` vault smart-contract, migrationg that token to the ETH address provided.
/// Different to the other debug `migrate` function is that this function will NOT update the
/// vault contract stored in the encrypted database.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// This function outputs a signed transaction which if NOT broadcast will result in the enclave no
/// longer working.  Use with extreme caution and only if you know exactly what you are doing!
#[named]
pub fn debug_get_erc20_on_evm_vault_single_migration_tx<D: DatabaseInterface>(
    db: &D,
    migrate_to_address_str: &str,
    token_address_str: &str,
    signature: &str,
) -> Result<String> {
    db.start_transaction()?;
    info!("✔ Debug getting `ERC20-on-EVM` migration transaction...");
    let eth_db_utils = EthDbUtils::new(db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let token_address = convert_hex_to_eth_address(token_address_str)?;
    let migrate_to_address = convert_hex_to_eth_address(migrate_to_address_str)?;
    CoreType::check_is_initialized(db)
        .and_then(|_| get_debug_command_hash!(function_name!(), migrate_to_address_str, token_address_str)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_migrate_single_fxn_data(&migrate_to_address, &token_address))
        .and_then(|tx_data| {
            let chain_id = eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                &chain_id,
                chain_id.get_erc20_vault_migrate_gas_limit(),
                eth_db_utils.get_eth_gas_price_from_db()?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&eth_db_utils.get_eth_private_key_from_db()?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            db.end_transaction()?;
            Ok(json!({
                "success": true,
                "eth_signed_tx": hex_tx,
                "token_address": format!("0x{}", hex::encode(token_address.as_bytes())),
                "migrated_to_address:": format!("0x{}", hex::encode(migrate_to_address.as_bytes())),
            })
            .to_string())
        })
}
