use crate::{chains::algo::algo_state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn parse_algo_submission_material_and_put_in_state<'a, D: DatabaseInterface>(
    algo_block_json: &str,
    state: AlgoState<'a, D>,
) -> Result<AlgoState<'a, D>> {
    unimplemented!();
    Ok(state)
}
