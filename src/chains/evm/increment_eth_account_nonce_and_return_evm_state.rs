use crate::{
    chains::{
        eth::{eth_database_utils_redux::EthDatabaseUtils, increment_eth_account_nonce::increment_eth_account_nonce},
        evm::eth_state::EthState as EvmState,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_increment_eth_account_nonce_and_return_evm_state<D: DatabaseInterface>(
    state: EvmState<D>,
) -> Result<EvmState<D>> {
    let num_txs = state.erc20_on_evm_eth_signed_txs.len();
    if num_txs == 0 {
        info!("✔ No signatures in state ∴ not incrementing ETH account nonce");
        Ok(state)
    } else {
        let eth_db_utils = EthDatabaseUtils::new(state.db); // FIXME Get from state eventually!
        increment_eth_account_nonce(
            &eth_db_utils,
            eth_db_utils.get_eth_account_nonce_from_db()?,
            num_txs as u64,
        )
        .and(Ok(state))
    }
}
