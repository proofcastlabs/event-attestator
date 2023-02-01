use crate::{
    chains::{btc::btc_state::BtcState, eth::eth_database_utils::EthDbUtilsExt},
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_increment_any_sender_nonce_in_db<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    if !state.use_any_sender_tx_type() {
        info!("✔ Not incrementing AnySender nonce - not an AnySender transaction!");
        return Ok(state);
    }
    match state.get_eth_signed_txs() {
        Err(_) => {
            info!("✔ Not incrementing AnySender nonce - no signatures made!");
            Ok(state)
        },
        Ok(signed_txs) => {
            info!("✔ Incrementing AnySender nonce by {}", signed_txs.len());
            state
                .eth_db_utils
                .increment_any_sender_nonce_in_db(signed_txs.len() as u64)
                .map(|_| state)
        },
    }
}
