use ethereum_types::Address as EthAddress;

use crate::{
    btc_on_int::check_core_is_initialized::check_core_is_initialized,
    chains::{
        btc::btc_database_utils::BtcDbUtils,
        eth::{
            eth_contracts::{
                erc777_proxy::{
                    get_signed_erc777_proxy_change_pnetwork_by_proxy_tx,
                    get_signed_erc777_proxy_change_pnetwork_tx,
                },
                erc777_token::get_signed_erc777_change_pnetwork_tx,
            },
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        },
    },
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Get Signed ERC777 change pNetwork Tx
///
/// This function will create a signed ETH transaction that will change the pNetwork address in
/// the pToken ERC777 contract to the passed in address.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, future ETH transactions will
/// fail due to the nonce being too high!
pub fn debug_get_signed_erc777_change_pnetwork_tx<D: DatabaseInterface>(
    db: &D,
    new_address: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(db);
    check_core_is_initialized(&eth_db_utils, &BtcDbUtils::new(db))
        .and_then(|_| check_debug_mode())
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
        .and_then(|_| {
            get_signed_erc777_change_pnetwork_tx(&eth_db_utils, EthAddress::from_slice(&hex::decode(new_address)?))
        })
        .and_then(|signed_tx_hex| {
            db.end_transaction()?;
            Ok(format!("{{signed_tx:{}}}", signed_tx_hex))
        })
        .map(prepend_debug_output_marker_to_string)
}

fn check_erc777_proxy_address_is_set<D: DatabaseInterface>(db: &D) -> Result<()> {
    info!("✔ Checking if the ERC777 proxy address is set...");
    check_debug_mode()
        .and_then(|_| EthDbUtils::new(db).get_erc777_proxy_contract_address_from_db())
        .and_then(|address| {
            if address.is_zero() {
                Err("✘ No ERC777 proxy address set in db - not signing tx!".into())
            } else {
                Ok(())
            }
        })
}

/// # Debug Get Signed ERC777 change pNetwork Tx
///
/// This function will create a signed ETH transaction that will change the pNetwork address in
/// the pToken ERC777 proxy contract to the passed in address.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, future ETH transactions will
/// fail due to the nonce being too high!
pub fn debug_get_signed_erc777_proxy_change_pnetwork_tx<D: DatabaseInterface>(
    db: &D,
    new_address: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(db);
    check_core_is_initialized(&eth_db_utils, &BtcDbUtils::new(db))
        .and_then(|_| check_debug_mode())
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
        .and_then(|_| check_erc777_proxy_address_is_set(db))
        .and_then(|_| {
            get_signed_erc777_proxy_change_pnetwork_tx(
                &eth_db_utils,
                EthAddress::from_slice(&hex::decode(new_address)?),
            )
        })
        .and_then(|signed_tx_hex| {
            db.end_transaction()?;
            Ok(format!("{{signed_tx:{}}}", signed_tx_hex))
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Get Signed ERC777 change pNetwork By Proxy Tx
///
/// This function will create a signed ETH transaction that will change the pNetwork address in
/// the pToken ERC777 contract via the ERC777 proxy contract, to the passed in address.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, future ETH transactions will
/// fail due to the nonce being too high!
pub fn debug_get_signed_erc777_proxy_change_pnetwork_by_proxy_tx<D: DatabaseInterface>(
    db: &D,
    new_address: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(db);
    check_core_is_initialized(&eth_db_utils, &BtcDbUtils::new(db))
        .and_then(|_| check_debug_mode())
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
        .and_then(|_| check_erc777_proxy_address_is_set(db))
        .and_then(|_| {
            get_signed_erc777_proxy_change_pnetwork_by_proxy_tx(
                &eth_db_utils,
                EthAddress::from_slice(&hex::decode(new_address)?),
            )
        })
        .and_then(|signed_tx_hex| {
            db.end_transaction()?;
            Ok(format!("{{signed_tx:{}}}", signed_tx_hex))
        })
        .map(prepend_debug_output_marker_to_string)
}
