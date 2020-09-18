use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::eth::eth_state::EthState,
    chains::eth::update_eth_tail_block_hash::maybe_update_eth_tail_block_hash,
};

pub fn maybe_update_eth_tail_block_hash_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe updating ETH tail block hash...");
    maybe_update_eth_tail_block_hash(&state.db).and(Ok(state))
}
