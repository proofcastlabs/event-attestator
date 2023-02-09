use common::{
    chains::{btc::btc_database_utils::BtcDbUtils, eos::eos_database_utils::EosDbUtils},
    constants::MAX_DATA_SENSITIVITY_LEVEL,
    core_type::CoreType,
    traits::DatabaseInterface,
    types::{Byte, Result},
    utils::get_prefixed_db_key,
};
use debug_signers::validate_debug_command_signature;
use function_name::named;

fn is_private_key_key<D: DatabaseInterface>(db: &D, key: &[Byte]) -> bool {
    key == get_prefixed_db_key("eth-private-key-key").to_vec()
        || key == get_prefixed_db_key("evm-private-key-key").to_vec()
        || key == get_prefixed_db_key("algo_private_key_key").to_vec()
        || key == BtcDbUtils::new(db).get_btc_private_key_db_key()
        || key == EosDbUtils::new(db).get_eos_private_key_db_key()
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
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
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
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash, cfg!(test)))
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
