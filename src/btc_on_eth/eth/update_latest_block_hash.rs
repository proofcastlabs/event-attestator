use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::eth::eth_state::EthState,
    chains::eth::update_latest_block_hash::update_latest_block_hash_if_subsequent,
};

pub fn maybe_update_latest_block_hash_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe updating latest ETH block hash if subsequent...");
    update_latest_block_hash_if_subsequent(&state.db, &state.get_eth_block_and_receipts()?.block).and(Ok(state))
}
