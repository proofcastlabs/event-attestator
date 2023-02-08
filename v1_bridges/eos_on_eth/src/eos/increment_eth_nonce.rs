use common::{
    chains::eth::eth_database_utils::{EthDbUtils, EthDbUtilsExt},
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_increment_eth_nonce_in_db_and_return_eos_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    let num_txs = state.eth_signed_txs.len() as u64;
    if num_txs == 0 {
        info!("✔ Not incrementing ETH account nonce - no signatures made!");
        Ok(state)
    } else {
        info!("✔ Incrementing ETH account nonce by {}", num_txs);
        EthDbUtils::new(state.db)
            .increment_eth_account_nonce_in_db(num_txs)
            .and(Ok(state))
    }
}
