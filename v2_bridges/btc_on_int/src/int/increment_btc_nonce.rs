use common::{chains::eth::EthState, traits::DatabaseInterface, types::Result};

pub fn maybe_increment_btc_account_nonce_and_return_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    match &state.btc_transactions {
        None => {
            info!("✔ Not incrementing BTC account nonce - no signatures made!");
            Ok(state)
        },
        Some(signed_txs) => {
            let current_nonce = state.btc_db_utils.get_btc_account_nonce_from_db()?;
            let num_txs = signed_txs.len();
            let new_nonce = num_txs as u64 + current_nonce;
            info!("✔ Incrementing btc account nonce by {num_txs} nonce from {current_nonce} to {new_nonce}");
            state.btc_db_utils.put_btc_account_nonce_in_db(new_nonce).and(Ok(state))
        },
    }
}
