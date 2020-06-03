#![allow(dead_code)]
use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::{
        eos_database_utils::put_eos_schedule_in_db,
        parse_eos_schedule::parse_schedule_string_to_schedule,
    },
};

fn add_new_schedule_to_db<D>(
    db: D,
    schedule_json: String,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Adding new EOS producer schedule to db...");
    parse_schedule_string_to_schedule(&schedule_json)
        .and_then(|schedule| put_eos_schedule_in_db(&db, &schedule))
        .map(|_| "{add_eos_schedule_sucesss:true}".to_string())
}
