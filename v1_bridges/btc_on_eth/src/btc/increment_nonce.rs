use common::{traits::DatabaseInterface, types::Result};
use common_btc::BtcState;
use common_eth::{EthDbUtils, EthDbUtilsExt, EthTransactions};

pub fn maybe_increment_nonce_in_db<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    if state.eth_signed_txs.is_empty() {
        info!("✔ No transactions in state so not incrementing any nonces!");
        Ok(state)
    } else {
        EthTransactions::from_bytes(&state.eth_signed_txs).and_then(|signed_txs| {
            let num_txs = signed_txs.len() as u64;
            let db_utils = EthDbUtils::new(state.db);
            if state.use_any_sender_tx_type() {
                info!("✔ Incrementing ETH account nonce by {num_txs}");
                db_utils.increment_any_sender_nonce_in_db(num_txs).and(Ok(state))
            } else {
                info!("✔ Incrementing ANY SENDER account nonce by {num_txs}");
                db_utils.increment_eth_account_nonce_in_db(num_txs).and(Ok(state))
            }
        })
    }
}
