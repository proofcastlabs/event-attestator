use crate::{
    chains::btc::filter_btc_txs::maybe_filter_out_btc_txs_with_too_many_outputs,
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_filter_btc_txs_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    let txs = state.btc_on_eos_signed_txs.clone();
    Ok(state.replace_btc_on_eos_signed_txs(maybe_filter_out_btc_txs_with_too_many_outputs(&txs)))
}
