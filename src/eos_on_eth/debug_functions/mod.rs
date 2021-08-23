pub(crate) mod block_reprocessors;

use std::str::FromStr;

use eos_chain::{AccountName as EosAccountName, Action as EosAction, PermissionLevel, Transaction as EosTransaction};
use ethereum_types::U256;
use serde_json::json;

use crate::{
    chains::{
        eos::{
            core_initialization::eos_init_utils::EosInitJson,
            eos_actions::PTokenPegOutAction,
            eos_constants::{
                get_eos_constants_db_keys,
                EOS_ACCOUNT_PERMISSION_LEVEL,
                EOS_PRIVATE_KEY_DB_KEY,
                PEGOUT_ACTION_NAME,
            },
            eos_crypto::{eos_private_key::EosPrivateKey, eos_transaction::EosSignedTransaction},
            eos_database_utils::{get_eos_account_name_from_db, get_eos_chain_id_from_db},
            eos_debug_functions::{
                add_eos_eth_token_dictionary_entry,
                add_new_eos_schedule,
                remove_eos_eth_token_dictionary_entry,
                update_incremerkle,
            },
            eos_utils::get_eos_tx_expiration_timestamp_with_offset,
        },
        eth::{
            eth_constants::{get_eth_constants_db_keys, ETH_PRIVATE_KEY_DB_KEY},
            eth_database_utils_redux::EthDatabaseUtils,
            eth_debug_functions::debug_set_eth_gas_price_in_db,
            eth_utils::convert_hex_to_eth_address,
        },
    },
    check_debug_mode::check_debug_mode,
    constants::{DB_KEY_PREFIX, MAX_DATA_SENSITIVITY_LEVEL},
    debug_database_utils::{get_key_from_db, set_key_in_db_to_value},
    dictionaries::{dictionary_constants::EOS_ETH_DICTIONARY_KEY, eos_eth::EosEthTokenDictionary},
    eos_on_eth::check_core_is_initialized::check_core_is_initialized,
    fees::fee_utils::sanity_check_basis_points_value,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Update Incremerkle
///
/// This function will take an EOS initialization JSON as its input and use it to create an
/// incremerkle valid for the block number in the JSON. It will then REPLACE the incremerkle in the
/// encrypted database with this one.
///
/// ### BEWARE:
/// Changing the incremerkle changes the last block the enclave has seen and so can easily lead to
/// transaction replays. Use with extreme caution and only if you know exactly what you are doing
/// and why.
pub fn debug_update_incremerkle<D: DatabaseInterface>(db: &D, eos_init_json: &str) -> Result<String> {
    check_core_is_initialized(&EthDatabaseUtils::new(db), &db)
        .and_then(|_| update_incremerkle(db, &EosInitJson::from_json_string(eos_init_json)?))
}

/// # Debug Add New Eos Schedule
///
/// Adds a new EOS schedule to the core's encrypted database.
pub fn debug_add_new_eos_schedule<D: DatabaseInterface>(db: D, schedule_json: &str) -> Result<String> {
    check_core_is_initialized(&EthDatabaseUtils::new(&db), &db).and_then(|_| add_new_eos_schedule(&db, schedule_json))
}

/// # Debug Set Key in DB to Value
///
/// This function set to the given value a given key in the encryped database.
///
/// ### BEWARE:
/// Only use this if you know exactly what you are doing and why.
pub fn debug_set_key_in_db_to_value<D: DatabaseInterface>(db: D, key: &str, value: &str) -> Result<String> {
    let key_bytes = hex::decode(&key)?;
    let is_private_key =
        { key_bytes == EOS_PRIVATE_KEY_DB_KEY.to_vec() || key_bytes == ETH_PRIVATE_KEY_DB_KEY.to_vec() };
    let sensitivity = if is_private_key {
        MAX_DATA_SENSITIVITY_LEVEL
    } else {
        None
    };
    set_key_in_db_to_value(db, key, value, sensitivity).map(prepend_debug_output_marker_to_string)
}

/// # Debug Get Key From Db
///
/// This function will return the value stored under a given key in the encrypted database.
pub fn debug_get_key_from_db<D: DatabaseInterface>(db: D, key: &str) -> Result<String> {
    let key_bytes = hex::decode(&key)?;
    let is_private_key =
        { key_bytes == EOS_PRIVATE_KEY_DB_KEY.to_vec() || key_bytes == ETH_PRIVATE_KEY_DB_KEY.to_vec() };
    let sensitivity = match is_private_key {
        true => MAX_DATA_SENSITIVITY_LEVEL,
        false => None,
    };
    get_key_from_db(db, key, sensitivity).map(prepend_debug_output_marker_to_string)
}

/// # Debug Get All Db Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    check_debug_mode().and(Ok(json!({
        "eth": get_eth_constants_db_keys(),
        "eos": get_eos_constants_db_keys(),
        "db-key-prefix": DB_KEY_PREFIX.to_string(),
        "dictionary:": hex::encode(EOS_ETH_DICTIONARY_KEY.to_vec()),
    })
    .to_string()))
}

/// # Debug Add ERC20 Dictionary Entry
///
/// This function will add an entry to the `EosEthTokenDictionary` held in the encrypted database. The
/// dictionary defines the relationship between ERC20 etheruem addresses and their pToken EOS
/// address counterparts.
///
/// The required format of an entry is:
/// {
///     "eos_symbol": <symbol>,
///     "eth_symbol": <symbol>,
///     "eos_address": <address>,
///     "eth_address": <address>,
///     "eth_token_decimals": <num-decimals>,
///     "eos_token_decimals": <num-decimals>,
/// }
pub fn debug_add_eos_eth_token_dictionary_entry<D: DatabaseInterface>(
    db: D,
    dictionary_entry_json_string: &str,
) -> Result<String> {
    check_core_is_initialized(&EthDatabaseUtils::new(&db), &db)
        .and_then(|_| add_eos_eth_token_dictionary_entry(&db, dictionary_entry_json_string))
}

/// # Debug Remove ERC20 Dictionary Entry
///
/// This function will remove an entry pertaining to the passed in ETH address from the
/// `EosEthTokenDictionary` held in the encrypted database, should that entry exist. If it is
/// not extant, nothing is changed.
pub fn debug_remove_eos_eth_token_dictionary_entry<D: DatabaseInterface>(
    db: D,
    eth_address_str: &str,
) -> Result<String> {
    check_core_is_initialized(&EthDatabaseUtils::new(&db), &db)
        .and_then(|_| remove_eos_eth_token_dictionary_entry(&db, eth_address_str))
}

/// # Debug Set ETH Gas Price
///
/// This function sets the ETH gas price to use when making ETH transactions. It's unit is `Wei`.
pub fn debug_set_eth_gas_price<D: DatabaseInterface>(db: D, gas_price: u64) -> Result<String> {
    debug_set_eth_gas_price_in_db(&db, gas_price)
}

/// # Debug Set ETH Fee Basis Points
///
/// This function takes an address and a new fee param. It gets the `EosEthTokenDictionary` from
/// the database then finds the entry pertaining to the address in question and if successful,
/// updates the fee associated with that address before saving the dictionary back into the
/// database. If no entry is found for a given `address` the function will return an error saying
/// as such.
///
/// #### NOTE: Using a fee of 0 will mean no fees are taken.
pub fn debug_set_eth_fee_basis_points<D: DatabaseInterface>(db: D, address: &str, new_fee: u64) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .map(|_| sanity_check_basis_points_value(new_fee))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EosEthTokenDictionary::get_from_db(&db))
        .and_then(|dictionary| {
            dictionary.change_eth_fee_basis_points_and_update_in_db(&db, &convert_hex_to_eth_address(address)?, new_fee)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true, "address": address, "new_fee": new_fee}).to_string())
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Set EOS Fee Basis Points
///
/// This function takes an address and a new fee param. It gets the `EosEthTokenDictionary` from
/// the database then finds the entry pertaining to the address in question and if successful,
/// updates the fee associated with that address before saving the dictionary back into the
/// database. If no entry is found for a given `address` the function will return an error saying
/// as such.
///
/// #### NOTE: Using a fee of 0 will mean no fees are taken.
pub fn debug_set_eos_fee_basis_points<D: DatabaseInterface>(db: D, address: &str, new_fee: u64) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .map(|_| sanity_check_basis_points_value(new_fee))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EosEthTokenDictionary::get_from_db(&db))
        .and_then(|dictionary| {
            dictionary.change_eos_fee_basis_points_and_update_in_db(&db, &EosAccountName::from_str(address)?, new_fee)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true, "address": address, "new_fee": new_fee}).to_string())
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Withwdraw Fees
///
/// This function takes an ETH address and uses it to search through the token dictionary to find a
/// corresponding entry. Once found, that entry's accrued fees are zeroed, a timestamp set in that
/// entry to mark the withdrawal date and the dictionary saved back in the database. Finally, an
/// EOS transaction is created to transfer the `<accrued_fees>` amount of tokens to the passed in
/// recipient address.
pub fn debug_withdraw_fees<D: DatabaseInterface>(
    db: D,
    token_address: &str,
    recipient_address: &str,
    ref_block_num: u16,
    ref_block_prefix: u32,
) -> Result<String> {
    let dictionary = EosEthTokenDictionary::get_from_db(&db)?;
    let dictionary_entry_eth_address = convert_hex_to_eth_address(token_address)?;
    let eos_smart_contract_address = get_eos_account_name_from_db(&db)?.to_string();
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| db.start_transaction())
        .and_then(|_| dictionary.withdraw_fees_and_save_in_db(&db, &dictionary_entry_eth_address))
        .and_then(|(_, fee_amount)| {
            let amount = dictionary.convert_u256_to_eos_asset_string(&dictionary_entry_eth_address, &fee_amount)?;
            info!("Amount as EOS asset: {}", amount);
            let eos_action = EosAction::from_str(
                &eos_smart_contract_address,
                &PEGOUT_ACTION_NAME.to_string(),
                vec![PermissionLevel::from_str(
                    &eos_smart_contract_address,
                    &EOS_ACCOUNT_PERMISSION_LEVEL.to_string(),
                )?],
                PTokenPegOutAction::from_str(
                    &dictionary
                        .get_entry_via_eth_address(&dictionary_entry_eth_address)?
                        .eos_address,
                    &amount,
                    recipient_address,
                    &[],
                )?,
            )?;
            EosSignedTransaction::from_unsigned_tx(
                &eos_smart_contract_address,
                &amount,
                &get_eos_chain_id_from_db(&db)?,
                &EosPrivateKey::get_from_db(&db)?,
                &EosTransaction::new(
                    get_eos_tx_expiration_timestamp_with_offset(0u32)?,
                    ref_block_num,
                    ref_block_prefix,
                    vec![eos_action],
                ),
            )
        })
        .and_then(|eos_signed_tx| {
            db.end_transaction()?;
            Ok(json!({
                "success": true,
                "eos_tx_signature": eos_signed_tx.signature,
                "eos_serialized_tx": eos_signed_tx.transaction,
            })
            .to_string())
        })
}

/// # Debug Set Accrued Fees
///
/// This function updates the accrued fees value in the dictionary entry retrieved from the passed
/// in ETH address.
pub fn debug_set_accrued_fees_in_dictionary<D: DatabaseInterface>(
    db: D,
    token_address: &str,
    fee_amount: String,
) -> Result<String> {
    info!("✔ Debug setting accrued fees in dictionary...");
    let dictionary = EosEthTokenDictionary::get_from_db(&db)?;
    let dictionary_entry_eth_address = convert_hex_to_eth_address(token_address)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| db.start_transaction())
        .and_then(|_| {
            dictionary.set_accrued_fees_and_save_in_db(
                &db,
                &dictionary_entry_eth_address,
                U256::from_dec_str(&fee_amount)?,
            )
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true,"fee":fee_amount}).to_string())
}
