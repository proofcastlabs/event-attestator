use common::{
    traits::{DatabaseInterface, Serdable},
    types::Result,
};
use common_btc::{BtcDbUtils, BtcTransactions};
use common_eth::EthState;

pub fn maybe_increment_btc_nonce_in_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if state.signed_txs.is_empty() {
        info!("✔ Not incrementing BTC account nonce - no signatures made!");
        Ok(state)
    } else {
        BtcTransactions::from_bytes(&state.signed_txs)
            .and_then(|signed_txs| {
                info!("✔ Incrementing BTC account nonce by {}", signed_txs.len());
                BtcDbUtils::new(state.db).increment_btc_account_nonce_in_db(signed_txs.len() as u64)
            })
            .and(Ok(state))
    }
}
