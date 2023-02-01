use crate::{
    chains::eth::eth_database_utils::{EthDbUtilsExt, EvmDbUtils},
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

fn increment_evm_account_nonce<D: DatabaseInterface>(db_utils: &EvmDbUtils<D>, num_signatures: u64) -> Result<()> {
    let current_nonce = db_utils.get_eth_account_nonce_from_db()?;
    let new_nonce = num_signatures + current_nonce;
    info!(
        "✔ Incrementing EVM account nonce by {} from {} to {}",
        num_signatures, current_nonce, new_nonce
    );
    db_utils.put_eth_account_nonce_in_db(new_nonce)
}

pub fn maybe_increment_evm_account_nonce_and_return_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    let num_txs = if !state.erc20_on_evm_evm_signed_txs.is_empty() {
        info!("✔ Found `erc20-on-evm` EVM signatures in state!");
        state.erc20_on_evm_evm_signed_txs.len()
    } else if !state.int_on_evm_evm_signed_txs.is_empty() {
        info!("✔ Found `int-on-evm` EVM signatures in state!");
        state.int_on_evm_evm_signed_txs.len()
    } else {
        0
    };

    debug!("Found {} txs!", num_txs);

    if num_txs == 0 {
        info!("✔ No signatures in state ∴ not incrementing EVM account nonce");
        Ok(state)
    } else {
        increment_evm_account_nonce(&state.evm_db_utils, num_txs as u64).and(Ok(state))
    }
}
