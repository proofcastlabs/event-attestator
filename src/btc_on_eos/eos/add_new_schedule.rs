use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::{
        eos_state::EosState,
        eos_database_utils::put_eos_schedule_in_db,
        parse_eos_schedule::parse_schedule_string_to_schedule,
    },
};

pub fn add_new_schedule_to_db<D>(
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

pub fn maybe_add_new_schedule_to_db<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe adding new EOS producer schedule to db...");
    match state.get_eos_block_header()?.new_producers.clone() {
        None => {
            trace!("✔ No new producer schedule in EOS block, doing nothing!");
            Ok(state)
        }
        Some(producer_schedule) => {
            info!("✔ New producer schedule found in block! Putting in db...");
            put_eos_schedule_in_db(&state.db, &producer_schedule)
                .and(Ok(state))
        }
    }
}
