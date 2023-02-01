use ethereum_types::Address as EthAddress;
use function_name::named;
use serde_json::json;

use crate::{
    btc_on_eth::constants::CORE_TYPE,
    chains::eth::{
        eth_chain_id::EthChainId,
        eth_crypto::eth_transaction::get_signed_minting_tx,
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
    },
    core_type::CoreType,
    debug_functions::validate_debug_command_signature,
    traits::DatabaseInterface,
    types::Result,
    utils::{decode_hex_with_err_msg, prepend_debug_output_marker_to_string, strip_hex_prefix},
};

/// # Debug Mint pBTC
///
/// This fxn simply creates & signs a pBTC minting transaction using the private key from the
/// database. It does __not__ change the database in __any way__, including incrementing the nonce
/// etc.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// There is great potential for bricking a running instance when using this, so only use it
/// if you know exactly what you're doing and why!
#[named]
pub fn debug_mint_pbtc<D: DatabaseInterface>(
    db: &D,
    amount: u128,
    nonce: u64,
    eth_network: &str,
    gas_price: u64,
    recipient: &str,
    signature: &str,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(db);

    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &amount, &nonce, eth_network, &gas_price, recipient)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| CoreType::check_is_initialized(db))
        .map(|_| strip_hex_prefix(recipient))
        .and_then(|hex_no_prefix| {
            decode_hex_with_err_msg(
                &hex_no_prefix,
                "Could not decode hex for recipient in `debug_mint_pbtc` fxn!",
            )
        })
        .map(|recipient_bytes| EthAddress::from_slice(&recipient_bytes))
        .and_then(|recipient_eth_address| {
            get_signed_minting_tx(
                &amount.into(),
                nonce,
                &EthChainId::from_str(eth_network)?,
                eth_db_utils.get_btc_on_eth_smart_contract_address_from_db()?,
                gas_price,
                &recipient_eth_address,
                &eth_db_utils.get_eth_private_key_from_db()?,
                None,
                None,
            )
        })
        .and_then(|signed_tx| {
            db.end_transaction()?;
            Ok(json!({
                "nonce": nonce,
                "amount": amount,
                "gas_price": gas_price,
                "recipient": recipient,
                "eth_network": eth_network,
                "signed_tx": signed_tx.serialize_hex(),
            })
            .to_string())
        })
        .map(prepend_debug_output_marker_to_string)
}
