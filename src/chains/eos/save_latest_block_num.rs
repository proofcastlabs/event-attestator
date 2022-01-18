use crate::{chains::eos::eos_state::EosState, traits::DatabaseInterface, types::Result};

pub fn save_latest_block_num_to_db<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Saving latest EOS block num in db...");
    state
        .eos_db_utils
        .put_eos_last_seen_block_num_in_db(state.get_eos_block_num()?)
        .and(Ok(state))
}
