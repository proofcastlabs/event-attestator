use crate::{
    types::Result,
    traits::DatabaseInterface,
    check_debug_mode::check_debug_mode,
    debug_database_utils::{
        get_key_from_db,
        set_key_in_db_to_value,
    },
    chains::btc::{
        btc_constants::BTC_PRIVATE_KEY_DB_KEY as BTC_KEY,
        utxo_manager::{
            debug_utxo_utils::clear_all_utxos,
            utxo_utils::get_all_utxos_as_json_string,
        },
    },
    btc_on_eos::{
        check_core_is_initialized::check_core_is_initialized,
        eos::{
            eos_database_utils::put_eos_schedule_in_db,
            eos_constants::EOS_PRIVATE_KEY_DB_KEY as EOS_KEY,
            parse_eos_schedule::parse_schedule_string_to_schedule,
        },
    },
};

pub fn debug_clear_all_utxos<D>(
    db: &D,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Debug clearing all UTXOs...");
    check_core_is_initialized(db)
        .and_then(|_| clear_all_utxos(db))
}

pub fn debug_add_new_eos_schedule<D>(
    db: D,
    schedule_json: String
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Debug adding new EOS schedule...");
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| db.start_transaction())
        .and_then(|_| parse_schedule_string_to_schedule(&schedule_json))
        .and_then(|schedule| put_eos_schedule_in_db(&db, &schedule))
        .and_then(|_| db.end_transaction())
        .map(|_| "{debug_adding_eos_schedule_succeeded:true}".to_string())
}

pub fn debug_set_key_in_db_to_value<D>(
    db: D,
    key: String,
    value: String
) -> Result<String>
    where D: DatabaseInterface
{
    check_core_is_initialized(&db)
        .and_then(|_| set_key_in_db_to_value(db, key, value))
}

pub fn debug_get_key_from_db<D>(
    db: D,
    key: String
) -> Result<String>
    where D: DatabaseInterface
{
    let key_bytes = hex::decode(&key)?;
    check_core_is_initialized(&db)
        .and_then(|_| {
            if key_bytes == EOS_KEY || key_bytes == BTC_KEY {
                get_key_from_db(db, key, Some(255))
            } else {
                get_key_from_db(db, key, None)
            }
        })
}

pub fn debug_get_all_utxos<D>(
    db: D
) -> Result<String>
    where D: DatabaseInterface
{
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| get_all_utxos_as_json_string(db))
}
