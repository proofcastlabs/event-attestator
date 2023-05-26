use common::{traits::DatabaseInterface, types::Result};
use common_algorand::AlgoDbUtils;
use common_eth::EthState;

fn increment_algo_account_nonce<D: DatabaseInterface>(db_utils: &AlgoDbUtils<D>, num_signatures: u64) -> Result<()> {
    let current_nonce = db_utils.get_algo_account_nonce()?;
    let new_nonce = num_signatures + current_nonce;
    info!("✔ Incrementing ALGO account nonce by {num_signatures} from {current_nonce} to {new_nonce}!");
    db_utils.put_algo_account_nonce_in_db(new_nonce)
}

pub fn maybe_increment_algo_account_nonce_and_return_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe incrementing ALGO account nonce...");
    let num_txs = state.algo_signed_txs.len();
    if num_txs == 0 {
        info!("✔ No signatures in state ∴ not incrementing ALGO account nonce");
        Ok(state)
    } else {
        increment_algo_account_nonce(&AlgoDbUtils::new(state.db), num_txs as u64).and(Ok(state))
    }
}
