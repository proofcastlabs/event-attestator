use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eth::{
        eth_state::EthState,
        eth_database_utils::get_eth_canon_to_tip_length_from_db,
        update_eth_canon_block_hash::maybe_update_canon_block_hash,
    },
};

pub fn maybe_update_eth_canon_block_hash_and_return_state<D>(
    state: EthState<D>
) -> Result<EthState<D>> where D: DatabaseInterface {
    info!("✔ Maybe updating ETH canon block hash...");
    get_eth_canon_to_tip_length_from_db(&state.db)
        .and_then(|canon_to_tip_length| maybe_update_canon_block_hash(&state.db, canon_to_tip_length))
        .and(Ok(state))
}
