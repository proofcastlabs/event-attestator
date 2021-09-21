use crate::{
    chains::{btc::filter_btc_txs::maybe_filter_out_btc_txs_with_too_many_outputs, eth::eth_state::EthState},
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_filter_btc_txs_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    let txs = state.get_btc_transactions()?;
    state.replace_btc_transactions(maybe_filter_out_btc_txs_with_too_many_outputs(&txs))
}
