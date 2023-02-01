use crate::{
    chains::btc::utxo_manager::utxo_database_utils::save_utxos_to_db,
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_save_btc_utxos_to_db<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Maybe saving BTC UTXOs...");
    match state.get_btc_utxos_and_values() {
        Err(_) => {
            info!("✔ No BTC UTXOs in state to save!");
            Ok(state)
        },
        Ok(utxos) => save_utxos_to_db(state.db, utxos).map(|_| state),
    }
}
