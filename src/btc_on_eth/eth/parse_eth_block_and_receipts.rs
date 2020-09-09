use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eth::eth::eth_state::EthState,
    chains::eth::parse_eth_block_and_receipts::parse_eth_block_and_receipts,
};

pub fn parse_eth_block_and_receipts_and_put_in_state<D>(
    block_json: &str,
    state: EthState<D>,
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    parse_eth_block_and_receipts(&block_json).and_then(|result| state.add_eth_block_and_receipts(result))
}
