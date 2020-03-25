use crate::btc_on_eos::{
    types::Result,
    eos::eos_state::EosState,
    traits::DatabaseInterface,
    btc::btc_database_utils::{
        put_btc_account_nonce_in_db,
        get_btc_account_nonce_from_db,
    },
};

fn increment_signature_nonce<D>(
    db: &D,
    current_nonce: &u64,
    num_signatures: &u64,
) -> Result<()>
    where D: DatabaseInterface
{
    debug!(
        "✔ Incrementing signature nonce from {} to {}",
        current_nonce,
        num_signatures + current_nonce,
    );
    put_btc_account_nonce_in_db(db, &(current_nonce + num_signatures))
}

pub fn maybe_increment_signature_nonce<D>(
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
            increment_signature_nonce(
                &state.db,
                &get_btc_account_nonce_from_db(&state.db)?,
                &(*num_txs as u64),
            )
                .map(|_| state)
        }
    }
}
