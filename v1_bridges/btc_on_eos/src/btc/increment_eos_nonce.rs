use common::{
    state::BtcState,
    traits::{DatabaseInterface, Serdable},
    types::Result,
};
use common_eos::{EosDbUtils, EosSignedTransactions};

fn increment_eos_nonce<D: DatabaseInterface>(
    db_utils: &EosDbUtils<D>,
    current_nonce: u64,
    num_signatures: u64,
) -> Result<()> {
    debug!("✔ Incrementing EOS  nonce from {} to {}", current_nonce, num_signatures);
    db_utils.put_eos_account_nonce_in_db(current_nonce + num_signatures)
}

pub fn maybe_increment_eos_nonce<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    let eos_db_utils = EosDbUtils::new(state.db);
    let eos_txs = EosSignedTransactions::from_bytes(&state.eos_signed_txs)?;
    if eos_txs.is_empty() {
        info!("✔ No EOS signatures in state ∴ not incrementing nonce");
        Ok(state)
    } else {
        increment_eos_nonce(
            &eos_db_utils,
            eos_db_utils.get_eos_account_nonce_from_db()?,
            eos_txs.len() as u64,
        )
        .and(Ok(state))
    }
}
