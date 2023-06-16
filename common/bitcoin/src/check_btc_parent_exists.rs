use common::{traits::DatabaseInterface, types::Result};

use crate::BtcState;

pub fn check_for_parent_of_btc_block_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Checking BTC block's parent exists in database...");
    let sub_mat = state.get_btc_block_and_id()?;
    let n = sub_mat.height;
    let prev_hash = sub_mat.block.header.prev_blockhash;
    debug!("searching db for block under hash: {prev_hash}");
    if state.btc_db_utils.get_btc_block_from_db(&prev_hash).is_ok() {
        info!("✔ BTC block's parent exists in database!");
        Ok(state)
    } else {
        Err(format!("✘ BTC block Rejected - no parent for block {n} exists in database!").into())
    }
}
