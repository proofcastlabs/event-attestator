use std::str::FromStr;

use rust_algorand::AlgorandBlock;

use crate::{chains::algo::algo_state::AlgoState, traits::DatabaseInterface, types::Result};

pub fn parse_algo_submission_material_and_put_in_state<'a, D: DatabaseInterface>(
    algo_block_json_str: &str,
    state: AlgoState<'a, D>,
) -> Result<AlgoState<'a, D>> {
    state.add_submitted_algo_block(&AlgorandBlock::from_str(&algo_block_json_str)?)
}
