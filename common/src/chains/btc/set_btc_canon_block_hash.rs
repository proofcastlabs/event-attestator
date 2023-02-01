use crate::{state::BtcState, traits::DatabaseInterface, types::Result};

pub fn maybe_set_btc_canon_block_hash<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Checking BTC canon block hash is set in database...");
    match state
        .btc_db_utils
        .key_exists_in_db(&state.btc_db_utils.get_btc_canon_block_hash_key(), None)
    {
        true => {
            info!("✔ BTC canon block hash set in database!");
            Ok(state)
        },
        false => {
            info!("✔ Setting BTC canon block hash from block in state...");
            state
                .btc_db_utils
                .put_btc_canon_block_hash_in_db(&state.get_btc_block_and_id()?.id)
                .and(Ok(state))
        },
    }
}
