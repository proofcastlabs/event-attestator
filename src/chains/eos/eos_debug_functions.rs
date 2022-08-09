use function_name::named;
use serde_json::json;

use crate::{
    chains::{
        eos::{
            core_initialization::eos_init_utils::{
                generate_and_put_incremerkle_in_db,
                put_eos_latest_block_info_in_db,
                EosInitJson,
            },
            eos_database_utils::EosDbUtils,
            eos_global_sequences::{GlobalSequences, ProcessedGlobalSequences},
            eos_producer_schedule::EosProducerScheduleV2,
        },
        eth::eth_utils::get_eth_address_from_str,
    },
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryEntry},
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
#[named]
pub fn debug_update_incremerkle<D: DatabaseInterface>(
    db: &D,
    eos_init_json_str: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug updating blockroot merkle...");
    let init_json = EosInitJson::from_json_string(eos_init_json_str)?;
    let eos_db_utils = EosDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), &init_json, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| put_eos_latest_block_info_in_db(&eos_db_utils, &init_json.block))
        .and_then(|_| generate_and_put_incremerkle_in_db(&eos_db_utils, &init_json.blockroot_merkle))
        .and_then(|_| db.end_transaction())
        .and(Ok("{debug_update_blockroot_merkle_success:true}".to_string()))
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Add New Eos Schedule
///
/// Adds a new EOS schedule to the core's encrypted database.
#[named]
pub fn debug_add_new_eos_schedule<D: DatabaseInterface>(
    db: &D,
    schedule_json: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug adding new EOS schedule...");
    let schedule = EosProducerScheduleV2::from_json(schedule_json)?;
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), &schedule, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| EosDbUtils::new(db).put_eos_schedule_in_db(&schedule))
        .and_then(|_| db.end_transaction())
        .and(Ok("{debug_adding_eos_schedule_success:true}".to_string()))
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Add ERC20 Token Dictionary Entry
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
#[named]
pub fn debug_add_token_dictionary_entry<D: DatabaseInterface>(
    db: &D,
    dictionary_entry_json_string: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug adding entry to `EosEthTokenDictionary`...");
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), dictionary_entry_json_string, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| {
            EosEthTokenDictionary::get_from_db(db)?
                .add_and_update_in_db(EosEthTokenDictionaryEntry::from_str(dictionary_entry_json_string)?, db)
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"adding_dictionary_entry_sucess":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Remove ERC20 Token Dictionary Entry
///
/// This function will remove an entry pertaining to the passed in ETH address from the
/// `EosEthTokenDictionary` held in the encrypted database, should that entry exist. If it is
/// not extant, nothing is changed.
#[named]
pub fn debug_remove_token_dictionary_entry<D: DatabaseInterface>(
    db: &D,
    eth_address_str: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug removing entry from `EosEthTokenDictionary`...");
    let dictionary = EosEthTokenDictionary::get_from_db(db)?;
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), eth_address_str, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| get_eth_address_from_str(eth_address_str))
        .and_then(|eth_address| dictionary.remove_entry_via_eth_address_and_update_in_db(&eth_address, db))
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"removing_dictionary_entry_sucess":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}

/// Debug Add Global Sequence to Processed List
///
/// This function will add a global sequence to the list of processed ones stored in the encrypted
/// database. This will mean that the EOS action with that global sequence cannot be processed.
#[named]
pub fn debug_add_global_sequences_to_processed_list<D: DatabaseInterface>(
    db: &D,
    global_sequences_json: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug adding global sequences to processed list...");
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), global_sequences_json, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| {
            ProcessedGlobalSequences::add_global_sequences_to_list_in_db(
                db,
                &mut GlobalSequences::from_str(global_sequences_json)?,
            )
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"added_global_sequences_to_processed_list":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}

/// Debug Remove Global Sequence From Processed List
///
/// This function will remove a global sequence from the list of processed ones stored in the
/// encrypted database. This allows a debug user to override the replay protection this list
/// provides. Use with caution!
#[named]
pub fn debug_remove_global_sequences_from_processed_list<D: DatabaseInterface>(
    db: &D,
    global_sequences_json: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug adding global sequences to processed list...");
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), global_sequences_json, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| {
            ProcessedGlobalSequences::remove_global_sequences_from_list_in_db(
                db,
                &GlobalSequences::from_str(global_sequences_json)?,
            )
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(
            json!({"removed_global_sequences_to_processed_list":true}).to_string()
        ))
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Set EOS Account Nonce
///
/// This function set to the given value EOS account nonce in the encryped database.
#[named]
pub fn debug_set_eos_account_nonce<D: DatabaseInterface>(
    db: &D,
    new_nonce: u64,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug setting EOS account nonce...");
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), &new_nonce, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| EosDbUtils::new(db).put_eos_account_nonce_in_db(new_nonce))
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"set_eos_account_nonce":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}

#[cfg(all(test, feature = "debug"))]
mod tests {
    use super::*;
    use crate::{chains::eos::eos_database_utils::EosDbUtils, test_utils::get_test_database};

    #[test]
    fn should_set_eos_account_nonce() {
        let db = get_test_database();
        let db_utils = EosDbUtils::new(&db);
        let nonce = 6;
        db_utils.put_eos_account_nonce_in_db(nonce).unwrap();
        assert_eq!(db_utils.get_eos_account_nonce_from_db().unwrap(), nonce);
        let new_nonce = 4;
        // NOTE: The debug command validation is skipped during tests...
        debug_set_eos_account_nonce(&db, new_nonce, &CoreType::BtcOnEos, "").unwrap();
        assert_eq!(db_utils.get_eos_account_nonce_from_db().unwrap(), new_nonce);
    }
}
