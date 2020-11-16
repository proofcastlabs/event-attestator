use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::btc::{
        btc_state::BtcState,
        filter_utxos::filter_out_utxos_whose_value_is_too_low,
    },
};

pub fn filter_out_value_too_low_utxos_from_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Maybe filtering out any UTXOs below minimum # of Satoshis...");
    filter_out_utxos_whose_value_is_too_low(&state.utxos_and_values)
        .and_then(|utxos| state.replace_utxos_and_values(utxos))
}
