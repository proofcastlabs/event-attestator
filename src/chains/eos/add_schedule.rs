use crate::{chains::eos::eos_state::EosState, traits::DatabaseInterface, types::Result};

pub fn maybe_add_new_eos_schedule_to_db_and_return_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    match &state.get_eos_block_header()?.new_producer_schedule {
        None => {
            info!("✔ No new schedule in block ∴ nothing to add to db!");
            Ok(state)
        },
        Some(new_schedule) => {
            info!(
                "✔ New producers schedule version {} found in EOS block, adding to db...",
                new_schedule.version
            );
            state.eos_db_utils.put_eos_schedule_in_db(new_schedule).and(Ok(state))
        },
    }
}
