use crate::{chains::eos::eos_state::EosState, traits::DatabaseInterface, types::Result};

pub fn save_incremerkle_from_state_to_db<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Saving incremerkle from state to db...");
    state
        .eos_db_utils
        .put_incremerkle_in_db(&state.incremerkle)
        .and(Ok(state))
}
