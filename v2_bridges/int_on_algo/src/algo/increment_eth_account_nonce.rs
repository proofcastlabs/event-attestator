use algorand::AlgoState;
use common::{chains::eth::eth_database_utils::EthDbUtilsExt, traits::DatabaseInterface, types::Result};

pub fn maybe_increment_eth_account_nonce_and_return_algo_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    let num_txs = state.get_num_signed_txs() as u64;
    if num_txs > 0 {
        info!("✔ Incrementing ETH account nonce by {num_txs} and returning ALGO state...");
        state
            .eth_db_utils
            .increment_eth_account_nonce_in_db(num_txs)
            .and(Ok(state))
    } else {
        Ok(state)
    }
}
