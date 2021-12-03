use crate::{
    chains::eth::{
        eth_database_utils::{EthDbUtilsExt, EvmDbUtils},
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

// TODO combine these various EVM-type nonce incrementers and use an enum for the different types instead!

fn increment_int_account_nonce<D: DatabaseInterface>(db_utils: &EvmDbUtils<D>, num_signatures: u64) -> Result<()> {
    let current_nonce = db_utils.get_eth_account_nonce_from_db()?;
    let new_nonce = num_signatures + current_nonce;
    info!(
        "✔ Incrementing INT account nonce by {} from {} to {}",
        num_signatures, current_nonce, new_nonce
    );
    db_utils.put_eth_account_nonce_in_db(new_nonce)
}

pub fn maybe_increment_int_account_nonce_and_return_eth_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    let num_txs = state.erc20_on_int_int_signed_txs.len();
    if num_txs == 0 {
        info!("✔ No signatures in state ∴ not incrementing INT account nonce");
        Ok(state)
    } else {
        increment_int_account_nonce(&state.evm_db_utils, num_txs as u64).and(Ok(state))
    }
}
