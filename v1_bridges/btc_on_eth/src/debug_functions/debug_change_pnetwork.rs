use common::{
    chains::eth::{
        eth_contracts::{
            erc777_proxy::{
                get_signed_erc777_proxy_change_pnetwork_by_proxy_tx,
                get_signed_erc777_proxy_change_pnetwork_tx,
            },
            erc777_token::get_signed_erc777_change_pnetwork_tx,
        },
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
    },
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use ethereum_types::Address as EthAddress;
use function_name::named;

use crate::constants::CORE_TYPE;

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
#[named]
pub fn debug_get_signed_erc777_change_pnetwork_tx<D: DatabaseInterface>(
    db: &D,
    new_address: &str,
    signature: &str,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(db);

    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), new_address)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| CoreType::check_is_initialized(db))
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
    EthDbUtils::new(db)
        .get_erc777_proxy_contract_address_from_db()
        .and_then(|address| match address.is_zero() {
            true => Err("✘ No ERC777 proxy address set in db - not signing tx!".into()),
            false => Ok(()),
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
#[named]
pub fn debug_get_signed_erc777_proxy_change_pnetwork_tx<D: DatabaseInterface>(
    db: &D,
    new_address: &str,
    signature: &str,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), new_address)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
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
#[named]
pub fn debug_get_signed_erc777_proxy_change_pnetwork_by_proxy_tx<D: DatabaseInterface>(
    db: &D,
    new_address: &str,
    signature: &str,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), new_address)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| CoreType::check_is_initialized(db))
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
