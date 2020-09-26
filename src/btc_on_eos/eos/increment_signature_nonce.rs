use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::btc::btc_database_utils::get_btc_account_nonce_from_db,
    chains::{
        eos::eos_state::EosState,
        btc::increment_btc_account_nonce::increment_btc_account_nonce,
    },
};

pub fn maybe_increment_signature_nonce_and_return_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    let num_txs = &state.signed_txs.len();
    match num_txs {
        0 => {
            info!("✔ No signatures in state ∴ not incrementing nonce");
            Ok(state)
        }
        _ => {
            increment_btc_account_nonce(&state.db, get_btc_account_nonce_from_db(&state.db)?, *num_txs as u64)
                .and(Ok(state))
        }
    }
}
