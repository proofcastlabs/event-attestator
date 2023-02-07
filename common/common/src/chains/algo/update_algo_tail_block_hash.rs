use crate::{
    chains::algo::get_candidate_block_hash::maybe_get_new_candidate_block_hash,
    state::AlgoState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_update_algo_tail_block_hash_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Maybe updating ALGO tail block hash...");
    maybe_get_new_candidate_block_hash(
        &state.algo_db_utils.get_tail_submission_material()?,
        state.algo_db_utils.maybe_get_new_tail_block_candidate()?,
    )
    .and_then(|maybe_new_tail_block_hash| match maybe_new_tail_block_hash {
        None => {
            info!("✔ No new ALGO tail block candidate found ∴ not updating!");
            Ok(state)
        },
        Some(hash) => {
            info!("✔ New ALGO tail block candidate found ∴ updating...");
            state.algo_db_utils.put_tail_block_hash_in_db(&hash)?;
            Ok(state)
        },
    })
}
