use crate::{
    chains::{btc::btc_database_utils::BtcDbUtils, eth::EthState},
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

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

pub fn maybe_increment_btc_account_nonce_and_return_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    match &state.btc_transactions {
        None => {
            info!("✔ Not incrementing BTC account nonce - no signatures made!");
            Ok(state)
        },
        Some(signed_txs) => {
            info!("✔ Incrementing BTC account nonce by {}", signed_txs.len());
            increment_btc_account_nonce(
                &state.btc_db_utils,
                state.btc_db_utils.get_btc_account_nonce_from_db()?,
                signed_txs.len() as u64,
            )
            .and(Ok(state))
        },
    }
}
