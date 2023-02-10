use common::{chains::eos::EosState, traits::DatabaseInterface, types::Result};
use common_eth::{EthDbUtils, EthDbUtilsExt, EthTransactions};

pub fn maybe_increment_int_nonce_in_db_and_return_eos_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    let num_txs = EthTransactions::from_bytes(&state.eth_signed_txs)?.len() as u64;
    if num_txs == 0 {
        info!("✔ Not incrementing INT account nonce - no signatures made!");
        Ok(state)
    } else {
        info!("✔ Incrementing INT account nonce by {}", num_txs);
        EthDbUtils::new(state.db)
            .increment_eth_account_nonce_in_db(num_txs)
            .and(Ok(state))
    }
}
