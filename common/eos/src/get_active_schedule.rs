use common::{traits::DatabaseInterface, types::Result};

use crate::EosState;

pub fn get_active_schedule_from_db_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if cfg!(feature = "non-validating") {
        info!("✔ Not getting EOS active-schedule/producer-list ∵ core is NOT validating!");
        Ok(state)
    } else {
        info!("✔ Getting EOS active-schedule/producer-list and adding to state...");
        state
            .eos_db_utils
            .get_eos_schedule_from_db(state.get_eos_block_header()?.schedule_version)
            .and_then(|schedule| state.add_active_schedule(schedule))
    }
}
