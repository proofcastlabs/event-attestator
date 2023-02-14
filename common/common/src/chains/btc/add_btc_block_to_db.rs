use crate::{chains::btc::BtcState, traits::DatabaseInterface, types::Result};

pub fn maybe_add_btc_block_to_db<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Checking if BTC block is already in the db...");
    if state
        .btc_db_utils
        .btc_block_exists_in_db(&state.get_btc_block_and_id()?.id)
    {
        Err("✘ BTC Block Rejected - it's already in the db!".into())
    } else {
        let block = state.get_btc_block_in_db_format()?;
        info!("✔ BTC block not in db!");
        info!("✔ Adding BTC block to db: {:?}", block);
        state.btc_db_utils.put_btc_block_in_db(block).map(|_| {
            info!("✔ BTC block added to database!");
            state
        })
    }
}
