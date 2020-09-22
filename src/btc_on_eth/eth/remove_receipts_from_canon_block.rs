use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::{
        eth_state::EthState,
        remove_receipts_from_canon_block::remove_receipts_from_canon_block_and_save_in_db,
    },
};

pub fn maybe_remove_receipts_from_canon_block_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>>
    where D: DatabaseInterface
{
    info!("✔ Removing receipts from canon block...");
    remove_receipts_from_canon_block_and_save_in_db(&state.db).and(Ok(state))
}
