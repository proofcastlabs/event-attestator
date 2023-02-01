use crate::{state::BtcState, traits::DatabaseInterface, types::Result};

pub fn check_for_parent_of_btc_block_in_state<D>(state: BtcState<D>) -> Result<BtcState<D>>
where
    D: DatabaseInterface,
{
    info!("✔ Checking BTC block's parent exists in database...");
    if state
        .btc_db_utils
        .get_btc_block_from_db(&state.get_btc_block_and_id()?.block.header.prev_blockhash)
        .is_ok()
    {
        info!("✔ BTC block's parent exists in database!");
        Ok(state)
    } else {
        Err("✘ BTC block Rejected - no parent exists in database!".into())
    }
}
