use common::{
    traits::{DatabaseInterface, Serdable},
    types::Result,
};
use common_btc::{save_utxos_to_db, BtcUtxosAndValues};
use common_eos::EosState;

pub fn maybe_save_btc_utxos_to_db<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    if state.btc_utxos_and_values.is_empty() {
        info!("✔ No BTC UTXOs in state to save!");
        Ok(state)
    } else {
        info!("✔ Saving BTC UTXOs...");
        BtcUtxosAndValues::from_bytes(&state.btc_utxos_and_values)
            .and_then(|utxos| save_utxos_to_db(state.db, &utxos))
            .and(Ok(state))
    }
}
