use common::{
    traits::{DatabaseInterface, Serdable},
    types::Result,
};
use common_btc::{BtcDbUtils, BtcTransactions};
use common_eth::EthState;

pub fn maybe_increment_btc_account_nonce_and_return_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if state.signed_txs.is_empty() {
        info!("✔ Not incrementing BTC account nonce - no signatures made!");
        Ok(state)
    } else {
        let signed_txs = BtcTransactions::from_bytes(&state.signed_txs)?;
        let btc_db_utils = BtcDbUtils::new(state.db);
        let current_nonce = btc_db_utils.get_btc_account_nonce_from_db()?;
        let num_txs = signed_txs.len();
        let new_nonce = num_txs as u64 + current_nonce;
        info!("✔ Incrementing btc account nonce by {num_txs} nonce from {current_nonce} to {new_nonce}");
        btc_db_utils.put_btc_account_nonce_in_db(new_nonce).and(Ok(state))
    }
}
