use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::{
        eth_state::EthState,
        eth_database_utils::get_eth_tail_block_from_db,
        remove_old_eth_tail_block::remove_parents_if_not_anchor,
    },
};

pub fn maybe_remove_old_eth_tail_block_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe removing old ETH tail block...");
    get_eth_tail_block_from_db(&state.db)
        .and_then(|tail_block| remove_parents_if_not_anchor(&state.db, &tail_block))
        .and(Ok(state))
}
