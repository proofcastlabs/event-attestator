use function_name::named;
use serde_json::json;

use crate::{
    chains::eth::{
        eth_contracts::erc20_vault::encode_erc20_vault_migrate_single_fxn_data,
        eth_crypto::eth_transaction::EthTransaction,
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        eth_utils::convert_hex_to_eth_address,
    },
    core_type::CoreType,
    debug_functions::validate_debug_command_signature,
    erc20_on_eos::constants::CORE_TYPE,
    traits::DatabaseInterface,
    types::Result,
};

/// # Debug Get EthOnEvmVault Single Migration Transaction
///
/// This function will create and sign a transaction that calls the `migrateSingle` function on
/// the `pERC20-on-EOS` vault smart-contract, migrationg that token to the ETH address provided.
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
pub fn debug_get_erc20_vault_migrate_single_tx<D: DatabaseInterface>(
    db: &D,
    migrate_to_address_str: &str,
    token_address_str: &str,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug getting migration transaction...");
    let eth_db_utils = EthDbUtils::new(db);
    db.start_transaction()?;
    CoreType::check_is_initialized(db)?;
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let token_address = convert_hex_to_eth_address(token_address_str)?;
    let migrate_to_address = convert_hex_to_eth_address(migrate_to_address_str)?;

    get_debug_command_hash!(function_name!(), migrate_to_address_str, token_address_str)()
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_migrate_single_fxn_data(&migrate_to_address, &token_address))
        .and_then(|tx_data| {
            let chain_id = eth_db_utils.get_eth_chain_id_from_db()?;
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_erc20_on_eos_smart_contract_address_from_db()?,
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
