use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::{
        eth_state::EthState,
        add_block_and_receipts_to_db::add_block_and_receipts_to_db_if_not_extant,
    },
};

pub fn maybe_add_block_and_receipts_to_db_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe adding ETH block and receipts if not in db...");
    add_block_and_receipts_to_db_if_not_extant(&state.db, state.get_eth_block_and_receipts()?).and(Ok(state))
}
