use crate::{chains::btc::btc_state::BtcState, traits::DatabaseInterface, types::Result};

pub fn maybe_filter_out_value_too_low_btc_on_eth_eth_tx_infos_in_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    state
        .btc_on_eth_eth_tx_infos
        .filter_out_value_too_low()
        .and_then(|params| state.replace_btc_on_eth_eth_tx_infos(params))
}
