use crate::{
    chains::btc::{btc_constants::BTC_LATEST_BLOCK_HASH_KEY, btc_state::BtcState},
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_set_btc_latest_block_hash<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Checking BTC latest block hash is set in database...");
    match state
        .btc_db_utils
        .key_exists_in_db(&BTC_LATEST_BLOCK_HASH_KEY.to_vec(), None)
    {
        true => {
            info!("✔ BTC latest block hash set in database");
            Ok(state)
        },
        false => {
            info!("✔ Initializing BTC latest block hash from in block...");
            state
                .btc_db_utils
                .put_btc_latest_block_hash_in_db(&state.get_btc_block_and_id()?.id)
                .and(Ok(state))
        },
    }
}
