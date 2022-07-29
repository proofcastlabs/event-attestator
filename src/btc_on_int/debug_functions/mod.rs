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
            btc_database_utils::{end_btc_db_transaction, BtcDatabaseKeysJson, BtcDbUtils},
            btc_debug_functions::debug_put_btc_fee_in_db,
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
            utxo_manager::{
                debug_utxo_utils::{
                    add_multiple_utxos,
                    clear_all_utxos,
                    consolidate_utxos,
                    get_child_pays_for_parent_btc_tx,
                    remove_utxo,
                },
                utxo_constants::get_utxo_constants_db_keys,
                utxo_utils::get_all_utxos_as_json_string,
            },
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
            eth_database_utils::{EthDatabaseKeysJson, EthDbUtils, EthDbUtilsExt},
            eth_debug_functions::debug_set_eth_gas_price_in_db,
        },
    },
    constants::{DB_KEY_PREFIX, MAX_DATA_SENSITIVITY_LEVEL, SUCCESS_JSON},
    core_type::CoreType,
    debug_mode::{
        check_debug_mode,
        get_key_from_db,
        set_key_in_db_to_value,
        validate_debug_command_signature,
        DEBUG_SIGNATORIES_DB_KEY,
    },
    traits::DatabaseInterface,
    types::Result,
    utils::{decode_hex_with_err_msg, prepend_debug_output_marker_to_string, strip_hex_prefix},
};

/// # Debug Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    check_debug_mode().map(|_| {
        json!({
            "btc": BtcDatabaseKeysJson::new(),
            "eth": EthDatabaseKeysJson::new(),
            "db_key_prefix": DB_KEY_PREFIX.to_string(),
            "utxo_manager": get_utxo_constants_db_keys(),
            "debug_signatories": format!("0x{}", hex::encode(&*DEBUG_SIGNATORIES_DB_KEY)),
        })
        .to_string()
    })
}

/// # Debug Clear All UTXOS
///
/// This function will remove ALL UTXOS from the core's encrypted database
///
/// ### BEWARE:
/// Use with extreme caution, and only if you know exactly what you are doing and why.
pub fn debug_clear_all_utxos<D: DatabaseInterface>(
    db: &D,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug clearing all UTXOs...");
    check_core_is_initialized(&EthDbUtils::new(db), &BtcDbUtils::new(db))
        .and_then(|_| check_debug_mode())
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
        .and_then(|_| clear_all_utxos(db))
        .and_then(|_| db.end_transaction())
        .map(|_| SUCCESS_JSON.to_string())
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Set Key in DB to Value
///
/// This function set to the given value a given key in the encryped database.
///
/// ### BEWARE:
/// Only use this if you know exactly what you are doing and why.
pub fn debug_set_key_in_db_to_value<D: DatabaseInterface>(
    db: &D,
    key: &str,
    value: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let key_bytes = hex::decode(&key)?;
    let sensitivity = if key_bytes == EthDbUtils::new(db).get_eth_private_key_db_key()
        || key_bytes == BtcDbUtils::new(db).get_btc_private_key_db_key()
    {
        MAX_DATA_SENSITIVITY_LEVEL
    } else {
        None
    };
    set_key_in_db_to_value(
        db,
        key,
        value,
        sensitivity,
        &CoreType::BtcOnInt,
        signature,
        debug_command_hash,
    )
    .map(prepend_debug_output_marker_to_string)
}

/// # Debug Get Key From Db
///
/// This function will return the value stored under a given key in the encrypted database.
pub fn debug_get_key_from_db<D: DatabaseInterface>(
    db: &D,
    key: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let key_bytes = hex::decode(&key)?;
    let sensitivity = if key_bytes == EthDbUtils::new(db).get_eth_private_key_db_key()
        || key_bytes == BtcDbUtils::new(db).get_btc_private_key_db_key()
    {
        MAX_DATA_SENSITIVITY_LEVEL
    } else {
        None
    };
    get_key_from_db(db, key, sensitivity, &CoreType::BtcOnInt, signature, debug_command_hash)
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Get All UTXOs
///
/// This function will return a JSON containing all the UTXOs the encrypted database currently has.
pub fn debug_get_all_utxos<D: DatabaseInterface>(db: &D, signature: &str, debug_command_hash: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &BtcDbUtils::new(db)))
        .and_then(|_| {
            let result = get_all_utxos_as_json_string(db)?;
            db.end_transaction()?;
            Ok(result)
        })
}

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

/// # Debug Get Child-Pays-For-Parent BTC Transaction
///
/// This function attempts to find the UTXO via the passed in transaction hash and vOut values, and
/// upon success creates a transaction spending that UTXO, sending it entirely to itself minus the
/// passed in fee.
///
/// ### BEWARE:
/// This function spends UTXOs and outputs the signed transactions. If the output trnsaction is NOT
/// broadcast, the change output saved in the DB will NOT be spendable, leaving the enclave
/// bricked. Use ONLY if you know exactly what you're doing and why!
pub fn debug_get_child_pays_for_parent_btc_tx<D: DatabaseInterface>(
    db: &D,
    fee: u64,
    tx_id: &str,
    v_out: u32,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &BtcDbUtils::new(db)))
        .and_then(|_| {
            db.end_transaction()?;
            get_child_pays_for_parent_btc_tx(db, fee, tx_id, v_out).map(prepend_debug_output_marker_to_string)
        })
}

/// # Debug Consolidate Utxos
///
/// This function removes X number of UTXOs from the database then crafts them into a single
/// transcation to itself before returning the serialized output ready for broadcasting, thus
/// consolidating those X UTXOs into a single one.
///
/// ### BEWARE:
/// This function spends UTXOs and outputs a signed transaction. If the outputted transaction is NOT
/// broadcast, the consolidated  output saved in the DB will NOT be spendable, leaving the enclave
/// bricked. Use ONLY if you know exactly what you're doing and why!
pub fn debug_consolidate_utxos<D: DatabaseInterface>(
    db: &D,
    fee: u64,
    num_utxos: usize,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &BtcDbUtils::new(db)))
        .and_then(|_| {
            db.end_transaction()?;
            consolidate_utxos(db, fee, num_utxos).map(prepend_debug_output_marker_to_string)
        })
}

/// # Debug Remove UTXO
///
/// Pluck a UTXO from the UTXO set and discard it, locating it via its transaction ID and v-out values.
///
/// ### BEWARE:
/// Use ONLY if you know exactly what you're doing and why!
pub fn debug_remove_utxo<D: DatabaseInterface>(
    db: &D,
    tx_id: &str,
    v_out: u32,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &BtcDbUtils::new(db)))
        .and_then(|_| {
            db.end_transaction()?;
            remove_utxo(db, tx_id, v_out).map(prepend_debug_output_marker_to_string)
        })
}

/// # Debug Add Multiple Utxos
///
/// Add multiple UTXOs to the databsae. This function first checks if that UTXO already exists in
/// the encrypted database, skipping it if so.
///
/// ### NOTE:
///
/// This function takes as it's argument and valid JSON string in the format that the
/// `debug_get_all_utxos` returns. In this way, it's useful for migrating a UTXO set from one core
/// to another.
///
/// ### BEWARE:
/// Use ONLY if you know exactly what you're doing and why!
pub fn debug_add_multiple_utxos<D: DatabaseInterface>(
    db: &D,
    json_str: &str,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnInt, signature, debug_command_hash))
        .and_then(|_| add_multiple_utxos(db, json_str))
        .and_then(|output| {
            db.end_transaction()?;
            Ok(prepend_debug_output_marker_to_string(output))
        })
}

/// # Debug Set ETH Gas Price
///
/// This function sets the ETH gas price to use when making ETH transactions. It's unit is `Wei`.
pub fn debug_set_int_gas_price<D: DatabaseInterface>(
    db: &D,
    gas_price: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    debug_set_eth_gas_price_in_db(db, gas_price, &CoreType::BtcOnInt, signature, debug_command_hash)
}

/// # Debug Set BTC fee
///
/// This function sets the BTC fee to the given value. The unit is satoshis per byte.
pub fn debug_set_btc_fee<D: DatabaseInterface>(
    db: &D,
    fee: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    debug_put_btc_fee_in_db(db, fee, &CoreType::BtcOnInt, signature, debug_command_hash)
}
