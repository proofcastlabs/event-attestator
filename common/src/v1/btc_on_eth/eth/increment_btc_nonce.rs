use crate::{state::EthState, traits::DatabaseInterface, types::Result};

pub fn maybe_increment_btc_nonce_in_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    match &state.btc_transactions {
        None => {
            info!("✔ Not incrementing BTC account nonce - no signatures made!");
            Ok(state)
        },
        Some(signed_txs) => {
            info!("✔ Incrementing BTC account nonce by {}", signed_txs.len());
            state
                .btc_db_utils
                .increment_btc_account_nonce_in_db(signed_txs.len() as u64)
                .and(Ok(state))
        },
    }
}
