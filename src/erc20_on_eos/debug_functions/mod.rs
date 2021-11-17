pub(crate) mod block_reprocessors;

use std::str::FromStr;

use eos_chain::AccountName as EosAccountName;
use ethereum_types::U256;
use serde_json::json;

use crate::{
    chains::{
        eos::{
            core_initialization::eos_init_utils::EosInitJson,
            eos_constants::{get_eos_constants_db_keys, EOS_PRIVATE_KEY_DB_KEY},
            eos_debug_functions::{
                add_eos_eth_token_dictionary_entry,
                add_new_eos_schedule,
                get_processed_actions_list,
                remove_eos_eth_token_dictionary_entry,
                update_incremerkle,
            },
        },
        eth::{
            eth_constants::{get_eth_constants_db_keys, ETH_PRIVATE_KEY_DB_KEY},
            eth_contracts::erc20_vault::{
                encode_erc20_vault_add_supported_token_fx_data,
                encode_erc20_vault_migrate_fxn_data,
                encode_erc20_vault_peg_out_fxn_data_without_user_data,
                encode_erc20_vault_remove_supported_token_fx_data,
                ERC20_VAULT_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT,
                ERC20_VAULT_MIGRATE_GAS_LIMIT,
                ERC20_VAULT_PEGOUT_WITHOUT_USER_DATA_GAS_LIMIT,
            },
            eth_crypto::eth_transaction::EthTransaction,
            eth_database_utils::EthDatabaseUtils,
            eth_debug_functions::debug_set_eth_gas_price_in_db,
            eth_utils::{convert_hex_to_eth_address, get_eth_address_from_str},
        },
    },
    check_debug_mode::check_debug_mode,
    constants::{DB_KEY_PREFIX, MAX_DATA_SENSITIVITY_LEVEL},
    debug_database_utils::{get_key_from_db, set_key_in_db_to_value},
    dictionaries::{dictionary_constants::EOS_ETH_DICTIONARY_KEY, eos_eth::EosEthTokenDictionary},
    erc20_on_eos::check_core_is_initialized::check_core_is_initialized,
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
    check_core_is_initialized(&EthDatabaseUtils::new(db), db)
        .and_then(|_| update_incremerkle(db, &EosInitJson::from_json_string(eos_init_json)?))
        .map(prepend_debug_output_marker_to_string)
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
    let sensitivity = match is_private_key {
        true => MAX_DATA_SENSITIVITY_LEVEL,
        false => None,
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
        "dictionary": hex::encode(EOS_ETH_DICTIONARY_KEY.to_vec()),
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

/// # Debug Get ERC20_VAULT Migration Transaction
///
/// This function will create and sign a transaction that calls the `migrate` function on the
/// current `pERC20-on-EOS` smart-contract, migrationg it to the ETH address provided as an
/// argument. It then updates the smart-contract address stored in the encrypted database to that
/// new address.
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// This function outputs a signed transaction which if NOT broadcast will result in the enclave no
/// longer working.  Use with extreme caution and only if you know exactly what you are doing!
pub fn debug_get_erc20_vault_migration_tx<D: DatabaseInterface>(
    db: D,
    new_eos_erc20_smart_contract_address_string: &str,
) -> Result<String> {
    db.start_transaction()?;
    info!("✔ Debug getting migration transaction...");
    let eth_db_utils = EthDatabaseUtils::new(&db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let current_eos_erc20_smart_contract_address = eth_db_utils.get_erc20_on_eos_smart_contract_address_from_db()?;
    let new_eos_erc20_smart_contract_address = get_eth_address_from_str(new_eos_erc20_smart_contract_address_string)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &db))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| eth_db_utils.put_erc20_on_eos_smart_contract_address_in_db(&new_eos_erc20_smart_contract_address))
        .and_then(|_| encode_erc20_vault_migrate_fxn_data(new_eos_erc20_smart_contract_address))
        .and_then(|tx_data| {
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                current_eos_erc20_smart_contract_address,
                &eth_db_utils.get_eth_chain_id_from_db()?,
                ERC20_VAULT_MIGRATE_GAS_LIMIT,
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
                "migrated_to_address:": new_eos_erc20_smart_contract_address.to_string(),
            })
            .to_string())
        })
}

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
pub fn debug_get_add_supported_token_tx<D: DatabaseInterface>(db: D, eth_address_str: &str) -> Result<String> {
    info!("✔ Debug getting `addSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDatabaseUtils::new(&db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = get_eth_address_from_str(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &db))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_add_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_erc20_on_eos_smart_contract_address_from_db()?,
                &eth_db_utils.get_eth_chain_id_from_db()?,
                ERC20_VAULT_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT,
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
pub fn debug_get_remove_supported_token_tx<D: DatabaseInterface>(db: D, eth_address_str: &str) -> Result<String> {
    info!("✔ Debug getting `removeSupportedToken` contract tx...");
    db.start_transaction()?;
    let eth_db_utils = EthDatabaseUtils::new(&db);
    let current_eth_account_nonce = eth_db_utils.get_eth_account_nonce_from_db()?;
    let eth_address = get_eth_address_from_str(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &db))
        .and_then(|_| eth_db_utils.increment_eth_account_nonce_in_db(1))
        .and_then(|_| encode_erc20_vault_remove_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                eth_db_utils.get_erc20_on_eos_smart_contract_address_from_db()?,
                &eth_db_utils.get_eth_chain_id_from_db()?,
                ERC20_VAULT_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT,
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

/// # Debug Get Processed Actions List
///
/// This function returns the list of already-processed action global sequences in JSON format.
pub fn debug_get_processed_actions_list<D: DatabaseInterface>(db: &D) -> Result<String> {
    check_core_is_initialized(&EthDatabaseUtils::new(db), db).and_then(|_| get_processed_actions_list(db))
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
        .and_then(|_| check_core_is_initialized(&EthDatabaseUtils::new(&db), &db))
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
        .and_then(|_| check_core_is_initialized(&EthDatabaseUtils::new(&db), &db))
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
pub fn debug_withdraw_fees_and_save_in_db<D: DatabaseInterface>(
    db: D,
    token_address: &str,
    recipient_address: &str,
) -> Result<String> {
    let eth_db_utils = EthDatabaseUtils::new(&db);
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&eth_db_utils, &db))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EosEthTokenDictionary::get_from_db(&db))
        .and_then(|dictionary| {
            dictionary.withdraw_fees_and_save_in_db(&db, &convert_hex_to_eth_address(token_address)?)
        })
        .and_then(|(token_address, fee_amount)| {
            Ok(EthTransaction::new_unsigned(
                encode_erc20_vault_peg_out_fxn_data_without_user_data(
                    convert_hex_to_eth_address(recipient_address)?,
                    token_address,
                    fee_amount,
                )?,
                eth_db_utils.get_eth_account_nonce_from_db()?,
                0,
                eth_db_utils.get_erc20_on_eos_smart_contract_address_from_db()?,
                &eth_db_utils.get_eth_chain_id_from_db()?,
                ERC20_VAULT_PEGOUT_WITHOUT_USER_DATA_GAS_LIMIT,
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
        .and_then(|_| check_core_is_initialized(&EthDatabaseUtils::new_for_eth(&db), &db))
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
