use crate::{chains::btc::btc_database_utils::BtcDbUtils, state::BtcState, traits::DatabaseInterface, types::Result};

pub fn is_btc_anchor_block_hash_set<D: DatabaseInterface>(db_utils: &BtcDbUtils<D>) -> bool {
    db_utils.key_exists_in_db(&db_utils.get_btc_anchor_block_hash_key(), None)
}

pub fn maybe_set_btc_anchor_block_hash<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Checking BTC anchor block hash is set in database...");
    if is_btc_anchor_block_hash_set(&state.btc_db_utils) {
        info!("✔ BTC anchor block hash set in database");
        Ok(state)
    } else {
        info!("✔ Setting BTC anchor hash from block in state...");
        state
            .btc_db_utils
            .put_btc_anchor_block_hash_in_db(&state.get_btc_block_and_id()?.id)
            .and(Ok(state))
    }
}
