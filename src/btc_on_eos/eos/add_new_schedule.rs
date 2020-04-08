use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_database_utils::put_eos_schedule_in_db,
    },
};

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
