use ethereum_types::Address as EthAddress;
use serde_json::json;

pub(crate) mod btc_block_reprocessor;
pub(crate) mod int_block_reprocessor;

use crate::{
    btc_on_int::check_core_is_initialized::{
        check_core_is_initialized,
        check_core_is_initialized_and_return_btc_state,
    },
    chains::{
        btc::{
            btc_block::parse_btc_block_and_id_and_put_in_state,
            btc_database_utils::{end_btc_db_transaction, BtcDbUtils},
            btc_state::BtcState,
            btc_submission_material::parse_btc_submission_json_and_put_in_state,
            extract_utxos_from_p2pkh_txs::maybe_extract_utxos_from_p2pkh_txs_and_put_in_btc_state,
            extract_utxos_from_p2sh_txs::maybe_extract_utxos_from_p2sh_txs_and_put_in_state,
            filter_p2pkh_deposit_txs::filter_for_p2pkh_deposit_txs_including_change_outputs_and_add_to_state,
            filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
            filter_utxos::{filter_out_utxos_extant_in_db_from_state, filter_out_value_too_low_utxos_from_state},
            get_deposit_info_hash_map::get_deposit_info_hash_map_and_put_in_state,
            save_utxos_to_db::maybe_save_utxos_to_db,
            set_flags::set_any_sender_flag_in_state,
            validate_btc_block_header::validate_btc_block_header_in_state,
            validate_btc_merkle_root::validate_btc_merkle_root,
            validate_btc_proof_of_work::validate_proof_of_work_of_btc_block_in_state,
        },
        eth::{
            eth_chain_id::EthChainId,
            eth_contracts::{
                erc777_proxy::{
                    get_signed_erc777_proxy_change_pnetwork_by_proxy_tx,
                    get_signed_erc777_proxy_change_pnetwork_tx,
                },
                erc777_token::get_signed_erc777_change_pnetwork_tx,
            },
            eth_crypto::eth_transaction::get_signed_minting_tx,
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        },
    },
    constants::SUCCESS_JSON,
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    traits::DatabaseInterface,
    types::Result,
    utils::{decode_hex_with_err_msg, prepend_debug_output_marker_to_string, strip_hex_prefix},
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

/// # Debug Maybe Add UTXO To DB
///
/// This function accepts as its param BTC submission material, in which it inspects all the
/// transactions looking for any pertaining to the core's own public key, or deposit addresses
/// derived from it. Any it finds it will extract the UTXO from and add it to the encrypted
/// database. Note that this fxn WILL extract the enclave's own change UTXOs from blocks!
///
/// ### NOTE:
/// The core won't accept UTXOs it already has in its encrypted database.
pub fn debug_maybe_add_utxo_to_db<D: DatabaseInterface>(
    db: &D,
    btc_submission_material_json: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
        .and_then(|_| parse_btc_submission_json_and_put_in_state(btc_submission_material_json, BtcState::init(db)))
        .and_then(set_any_sender_flag_in_state)
        .and_then(parse_btc_block_and_id_and_put_in_state)
        .and_then(check_core_is_initialized_and_return_btc_state)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(filter_for_p2pkh_deposit_txs_including_change_outputs_and_add_to_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(maybe_extract_utxos_from_p2pkh_txs_and_put_in_btc_state)
        .and_then(maybe_extract_utxos_from_p2sh_txs_and_put_in_state)
        .and_then(filter_out_value_too_low_utxos_from_state)
        .and_then(filter_out_utxos_extant_in_db_from_state)
        .and_then(maybe_save_utxos_to_db)
        .and_then(end_btc_db_transaction)
        .map(|_| SUCCESS_JSON.to_string())
        .map(prepend_debug_output_marker_to_string)
}

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
pub fn debug_mint_pbtc<D: DatabaseInterface>(
    db: &D,
    amount: u128,
    nonce: u64,
    eth_network: &str,
    gas_price: u64,
    recipient: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(db);
    check_core_is_initialized(&eth_db_utils, &BtcDbUtils::new(db))
        .and_then(|_| check_debug_mode())
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
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
