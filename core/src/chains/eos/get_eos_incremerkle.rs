use crate::{state::EosState, traits::DatabaseInterface, types::Result};

pub fn get_incremerkle_and_add_to_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    state
        .eos_db_utils
        .get_incremerkle_from_db()
        .map(|incremerkle| state.add_incremerkle(incremerkle))
}
