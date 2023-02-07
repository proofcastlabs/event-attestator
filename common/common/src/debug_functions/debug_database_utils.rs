use function_name::named;

use crate::{
    chains::{
        btc::btc_database_utils::BtcDbUtils,
        eos::eos_database_utils::EosDbUtils,
        eth::eth_database_utils::{EthDbUtils, EvmDbUtils},
    },
    constants::MAX_DATA_SENSITIVITY_LEVEL,
    core_type::CoreType,
    debug_functions::validate_debug_command_signature,
    traits::DatabaseInterface,
    types::{Byte, Result},
    utils::get_prefixed_db_key,
};

fn is_private_key_key<D: DatabaseInterface>(db: &D, key: &[Byte]) -> bool {
    key == EthDbUtils::new(db).get_eth_private_key_db_key()
        || key == EvmDbUtils::new(db).get_evm_private_key_db_key()
        || key == BtcDbUtils::new(db).get_btc_private_key_db_key()
        || key == EosDbUtils::new(db).get_eos_private_key_db_key()
        || key == get_prefixed_db_key("algo_private_key_key").to_vec()
}
/// Debug Set Key In Db To Value
///
/// Sets a provide key to a provided value in the database.
#[named]
pub fn debug_set_key_in_db_to_value<D: DatabaseInterface>(
    db: &D,
    key: &str,
    value: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Setting key: {} in DB to value: {}", key, value);
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), key, value, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| {
            let key_bytes = hex::decode(key)?;
            let data_sensitivity = if is_private_key_key(db, &key_bytes) {
                MAX_DATA_SENSITIVITY_LEVEL
            } else {
                None
            };
            db.put(key_bytes, hex::decode(value)?, data_sensitivity)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| "{putting_value_in_database_suceeded:true}".to_string())
}

/// Debug Get Key From Db
///
/// Gets the value from the given key (if extant) from the database.
#[named]
pub fn debug_get_key_from_db<D: DatabaseInterface>(
    db: &D,
    key: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    info!("✔ Maybe getting key: {} from DB...", key);
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), key, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| {
            let key_bytes = hex::decode(key)?;
            let data_sensitivity = if is_private_key_key(db, &key_bytes) {
                MAX_DATA_SENSITIVITY_LEVEL
            } else {
                None
            };
            db.get(key_bytes, data_sensitivity)
        })
        .and_then(|value| {
            db.end_transaction()?;
            Ok(format!("{{key:{},value:{}}}", key, hex::encode(value)))
        })
}
