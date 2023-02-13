use common::{chains::btc::btc_database_utils::BtcDbUtils, traits::DatabaseInterface, types::Result};
use common_eos::EosState;

pub fn increment_btc_account_nonce<D: DatabaseInterface>(
    db_utils: &BtcDbUtils<D>,
    current_nonce: u64,
    num_signatures: u64,
) -> Result<()> {
    let new_nonce = num_signatures + current_nonce;
    info!(
        "✔ Incrementing btc account nonce by {} nonce from {} to {}",
        num_signatures, current_nonce, new_nonce
    );
    db_utils.put_btc_account_nonce_in_db(new_nonce)
}

pub fn maybe_increment_btc_signature_nonce_and_return_eos_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    let num_txs = state.btc_on_eos_signed_txs.len();
    if num_txs == 0 {
        info!("✔ No signatures in state ∴ not incrementing nonce");
        Ok(state)
    } else {
        increment_btc_account_nonce(
            &state.btc_db_utils,
            state.btc_db_utils.get_btc_account_nonce_from_db()?,
            num_txs as u64,
        )
        .and(Ok(state))
    }
}
