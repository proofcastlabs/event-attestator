use serde_json::json;

use crate::{
    chains::{
        eth::{
            eth_constants::{get_eth_constants_db_keys, ETH_PRIVATE_KEY_DB_KEY as ETH_KEY},
            eth_contracts::erc20_vault::{
                encode_erc20_vault_add_supported_token_fx_data,
                encode_erc20_vault_migrate_fxn_data,
                encode_erc20_vault_remove_supported_token_fx_data,
                ERC20_VAULT_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT,
                ERC20_VAULT_MIGRATE_GAS_LIMIT,
            },
            eth_crypto::eth_transaction::EthTransaction,
            eth_database_transactions::{
                end_eth_db_transaction_and_return_state,
                start_eth_db_transaction_and_return_state,
            },
            eth_database_utils::{
                get_eth_account_nonce_from_db,
                get_eth_chain_id_from_db,
                get_eth_gas_price_from_db,
                get_eth_on_evm_smart_contract_address_from_db,
                get_eth_private_key_from_db,
                get_latest_eth_block_number,
                increment_eth_account_nonce_in_db,
                put_eth_on_evm_smart_contract_address_in_db,
            },
            eth_state::EthState,
            eth_submission_material::parse_eth_submission_material_and_put_in_state,
            eth_utils::{convert_hex_to_address, get_eth_address_from_str},
            increment_evm_account_nonce::maybe_increment_evm_account_nonce_and_return_eth_state,
            validate_block_in_state::validate_block_in_state,
            validate_receipts_in_state::validate_receipts_in_state,
        },
        evm::{
            eth_constants::{
                get_eth_constants_db_keys as get_evm_constants_db_keys,
                ETH_PRIVATE_KEY_DB_KEY as EVM_KEY,
            },
            eth_database_transactions::{
                end_eth_db_transaction_and_return_state as end_evm_db_tx_and_return_state,
                start_eth_db_transaction_and_return_state as start_evm_db_tx_and_return_state,
            },
            eth_database_utils::{
                get_any_sender_nonce_from_db as get_evm_any_sender_nonce_from_db,
                get_eth_account_nonce_from_db as get_evm_account_nonce_from_db,
                get_latest_eth_block_number as get_latest_evm_block_number,
            },
            eth_state::EthState as EvmState,
            eth_submission_material::parse_eth_submission_material_and_put_in_state as parse_evm_submission_material_and_put_in_state,
            increment_eth_account_nonce_and_return_evm_state::maybe_increment_eth_account_nonce_and_return_evm_state,
            validate_block_in_state::validate_block_in_state as validate_evm_block_in_state,
            validate_receipts_in_state::validate_receipts_in_state as validate_evm_receipts_in_state,
        },
    },
    check_debug_mode::check_debug_mode,
    constants::{DB_KEY_PREFIX, PRIVATE_KEY_DATA_SENSITIVITY_LEVEL},
    debug_database_utils::{get_key_from_db, set_key_in_db_to_value},
    dictionaries::eth_evm::{
        get_eth_evm_token_dictionary_from_db_and_add_to_eth_state,
        get_eth_evm_token_dictionary_from_db_and_add_to_evm_state,
        EthEvmTokenDictionary,
        EthEvmTokenDictionaryEntry,
    },
    eth_on_evm::{
        check_core_is_initialized::{
            check_core_is_initialized,
            check_core_is_initialized_and_return_eth_state,
            check_core_is_initialized_and_return_evm_state,
        },
        eth::{
            evm_tx_info::{
                filter_out_zero_value_tx_infos_from_state,
                filter_submission_material_for_peg_in_events_in_state,
                maybe_sign_evm_txs_and_add_to_eth_state,
                EthOnEvmEvmTxInfos,
            },
            get_eth_output_json::{get_evm_signed_tx_info_from_evm_txs, EthOutput},
        },
        evm::{
            eth_tx_info::{
                filter_out_zero_value_tx_infos_from_state as filter_out_zero_value_eth_txs_from_state,
                filter_submission_material_for_redeem_events_in_state,
                maybe_sign_eth_txs_and_add_to_evm_state,
                EthOnEvmEthTxInfos,
            },
            get_evm_output_json::{get_eth_signed_tx_info_from_evm_txs, EvmOutput},
        },
    },
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Reprocess EVM Block
///
/// This function will take a passed in EVM block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block
///
/// ### NOTE:
/// This function will increment the core's EVM nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, ALL future EVM transactions will
/// fail due to the core having an incorret nonce!
pub fn debug_reprocess_evm_block<D: DatabaseInterface>(db: D, evm_block_json: &str) -> Result<String> {
    info!("✔ Submitting ETH block to core...");
    check_debug_mode()
        .and_then(|_| parse_evm_submission_material_and_put_in_state(evm_block_json, EvmState::init(db)))
        .and_then(check_core_is_initialized_and_return_evm_state)
        .and_then(start_evm_db_tx_and_return_state)
        .and_then(validate_evm_block_in_state)
        .and_then(validate_evm_receipts_in_state)
        .and_then(get_eth_evm_token_dictionary_from_db_and_add_to_evm_state)
        .and_then(filter_submission_material_for_redeem_events_in_state)
        .and_then(|state| {
            state
                .get_eth_submission_material()
                .and_then(|material| {
                    EthOnEvmEthTxInfos::from_submission_material(
                        &material,
                        &EthEvmTokenDictionary::get_from_db(&state.db)?,
                    )
                })
                .and_then(|params| state.add_eth_on_evm_eth_tx_infos(params))
        })
        .and_then(filter_out_zero_value_eth_txs_from_state)
        .and_then(maybe_sign_eth_txs_and_add_to_evm_state)
        .and_then(maybe_increment_eth_account_nonce_and_return_evm_state)
        .and_then(end_evm_db_tx_and_return_state)
        .and_then(|state| {
            info!("✔ Getting EVM output json...");
            let output = serde_json::to_string(&EvmOutput {
                evm_latest_block_number: get_latest_evm_block_number(&state.db)?,
                eth_signed_transactions: if state.eth_on_evm_eth_signed_txs.is_empty() {
                    vec![]
                } else {
                    let use_any_sender_tx = false;
                    get_eth_signed_tx_info_from_evm_txs(
                        &state.eth_on_evm_eth_signed_txs,
                        &state.eth_on_evm_eth_tx_infos,
                        get_evm_account_nonce_from_db(&state.db)?,
                        use_any_sender_tx,
                        get_evm_any_sender_nonce_from_db(&state.db)?,
                        get_latest_eth_block_number(&state.db)?,
                    )?
                },
            })?;
            info!("✔ Reprocess EVM block output: {}", output);
            Ok(output)
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Reprocess ETH Block
///
/// This function will take a passed in ETH block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block
///
/// ### NOTE:
/// This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, ALL future ETH transactions will
/// fail due to the core having an incorret nonce!
pub fn debug_reprocess_eth_block<D: DatabaseInterface>(db: D, eth_block_json: &str) -> Result<String> {
    info!("✔ Submitting ETH block to core...");
    check_debug_mode()
        .and_then(|_| parse_eth_submission_material_and_put_in_state(eth_block_json, EthState::init(db)))
        .and_then(check_core_is_initialized_and_return_eth_state)
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(get_eth_evm_token_dictionary_from_db_and_add_to_eth_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(|state| {
            state
                .get_eth_submission_material()
                .and_then(|material| {
                    EthOnEvmEvmTxInfos::from_submission_material(
                        &material,
                        &get_eth_on_evm_smart_contract_address_from_db(&state.db)?,
                        &EthEvmTokenDictionary::get_from_db(&state.db)?,
                    )
                })
                .and_then(|params| state.add_eth_on_evm_evm_tx_infos(params))
        })
        .and_then(filter_out_zero_value_tx_infos_from_state)
        .and_then(maybe_sign_evm_txs_and_add_to_eth_state)
        .and_then(maybe_increment_evm_account_nonce_and_return_eth_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(|state| {
            info!("✔ Getting ETH output json...");
            let output = serde_json::to_string(&EthOutput {
                eth_latest_block_number: get_latest_eth_block_number(&state.db)?,
                evm_signed_transactions: if state.eth_on_evm_evm_signed_txs.is_empty() {
                    vec![]
                } else {
                    get_evm_signed_tx_info_from_evm_txs(
                        &state.eth_on_evm_evm_signed_txs,
                        &state.eth_on_evm_evm_tx_infos,
                        get_evm_account_nonce_from_db(&state.db)?,
                        false, // TODO Get this from state submission material when/if we support AnySender
                        get_evm_any_sender_nonce_from_db(&state.db)?,
                        get_latest_evm_block_number(&state.db)?,
                    )?
                },
            })?;
            info!("✔ Reprocess ETH block output: {}", output);
            Ok(output)
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Get All DB Keys
///
/// This function will return a JSON formatted list of all the database keys used in the encrypted database.
pub fn debug_get_all_db_keys() -> Result<String> {
    check_debug_mode().map(|_| {
        json!({
            "evm": get_evm_constants_db_keys(),
            "eth": get_eth_constants_db_keys(),
            "db-key-prefix": DB_KEY_PREFIX.to_string(),
        })
        .to_string()
    })
}

/// # Debug Set Key in DB to Value
///
/// This function set to the given value a given key in the encryped database.
///
/// ### BEWARE:
/// Only use this if you know exactly what you are doing and why.
pub fn debug_set_key_in_db_to_value<D: DatabaseInterface>(db: D, key: &str, value: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| {
            let key_bytes = hex::decode(&key)?;
            let sensitivity = match key_bytes == ETH_KEY.to_vec() || key_bytes == EVM_KEY.to_vec() {
                true => PRIVATE_KEY_DATA_SENSITIVITY_LEVEL,
                false => None,
            };
            set_key_in_db_to_value(db, key, value, sensitivity)
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Get Key From Db
///
/// This function will return the value stored under a given key in the encrypted database.
pub fn debug_get_key_from_db<D: DatabaseInterface>(db: D, key: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| {
            let key_bytes = hex::decode(&key)?;
            let sensitivity = match key_bytes == ETH_KEY.to_vec() || key_bytes == EVM_KEY.to_vec() {
                true => PRIVATE_KEY_DATA_SENSITIVITY_LEVEL,
                false => None,
            };
            get_key_from_db(db, key, sensitivity)
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Add Dictionary Entry
///
/// This function will add an entry to the `EthEvmTokenDictionary` held in the encrypted database. The
/// dictionary defines the relationship between ETH token addresses and the address of their pTokenized,
/// EVM-compliant counterparts.
///
/// The required format of an entry is:
/// {
///     "eth_symbol": <symbol>,
///     "evm_symbol": <symbol>,
///     "eth_address": <address>,
///     "evm_address": <address>,
/// }
pub fn debug_add_dictionary_entry<D: DatabaseInterface>(db: D, json_str: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthEvmTokenDictionary::get_from_db(&db))
        .and_then(|dictionary| dictionary.add_and_update_in_db(EthEvmTokenDictionaryEntry::from_str(json_str)?, &db))
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"add_dictionary_entry_success:":"true"}).to_string())
}

/// # Debug Remove Dictionary Entry
///
/// This function will remove an entry pertaining to the passed in ETH address from the
/// `EthEvmTokenDictionaryEntry` held in the encrypted database, should that entry exist. If it is
/// not extant, nothing is changed.
pub fn debug_remove_dictionary_entry<D: DatabaseInterface>(db: D, eth_address_str: &str) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| db.start_transaction())
        .and_then(|_| EthEvmTokenDictionary::get_from_db(&db))
        .and_then(|dictionary| {
            dictionary.remove_entry_via_eth_address_and_update_in_db(&convert_hex_to_address(eth_address_str)?, &db)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"remove_dictionary_entry_success:":"true"}).to_string())
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
    let current_eth_account_nonce = get_eth_account_nonce_from_db(&db)?;
    let eth_address = convert_hex_to_address(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| increment_eth_account_nonce_in_db(&db, 1))
        .and_then(|_| encode_erc20_vault_add_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                get_eth_on_evm_smart_contract_address_from_db(&db)?,
                get_eth_chain_id_from_db(&db)?,
                ERC20_VAULT_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT,
                get_eth_gas_price_from_db(&db)?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&get_eth_private_key_from_db(&db)?))
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
    let current_eth_account_nonce = get_eth_account_nonce_from_db(&db)?;
    let eth_address = convert_hex_to_address(eth_address_str)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| increment_eth_account_nonce_in_db(&db, 1))
        .and_then(|_| encode_erc20_vault_remove_supported_token_fx_data(eth_address))
        .and_then(|tx_data| {
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                get_eth_on_evm_smart_contract_address_from_db(&db)?,
                get_eth_chain_id_from_db(&db)?,
                ERC20_VAULT_CHANGE_SUPPORTED_TOKEN_GAS_LIMIT,
                get_eth_gas_price_from_db(&db)?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&get_eth_private_key_from_db(&db)?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            db.end_transaction()?;
            Ok(json!({ "success": true, "eth_signed_tx": hex_tx }).to_string())
        })
}

/// # Debug Get EthOnEvmVault Migration Transaction
///
/// This function will create and sign a transaction that calls the `migrate` function on the
/// current `pETH-on-EVM` vault smart-contract, migrationg it to the ETH address provided as an
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
pub fn debug_get_eth_on_evm_vault_migration_tx<D: DatabaseInterface>(db: D, new_address: &str) -> Result<String> {
    db.start_transaction()?;
    info!("✔ Debug getting `ETH-on-EVM` migration transaction...");
    let current_eth_account_nonce = get_eth_account_nonce_from_db(&db)?;
    let current_smart_contract_address = get_eth_on_evm_smart_contract_address_from_db(&db)?;
    let new_smart_contract_address = get_eth_address_from_str(new_address)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| increment_eth_account_nonce_in_db(&db, 1))
        .and_then(|_| put_eth_on_evm_smart_contract_address_in_db(&db, &new_smart_contract_address))
        .and_then(|_| encode_erc20_vault_migrate_fxn_data(new_smart_contract_address))
        .and_then(|tx_data| {
            Ok(EthTransaction::new_unsigned(
                tx_data,
                current_eth_account_nonce,
                0,
                current_smart_contract_address,
                get_eth_chain_id_from_db(&db)?,
                ERC20_VAULT_MIGRATE_GAS_LIMIT,
                get_eth_gas_price_from_db(&db)?,
            ))
        })
        .and_then(|unsigned_tx| unsigned_tx.sign(&get_eth_private_key_from_db(&db)?))
        .map(|signed_tx| signed_tx.serialize_hex())
        .and_then(|hex_tx| {
            db.end_transaction()?;
            Ok(json!({
                "success": true,
                "eth_signed_tx": hex_tx,
                "migrated_to_address:": new_smart_contract_address.to_string(),
            })
            .to_string())
        })
}
