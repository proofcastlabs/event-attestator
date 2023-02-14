use common::{traits::DatabaseInterface, types::Result};

use crate::{utxo_manager::save_utxos_to_db, BtcState};

pub fn maybe_save_utxos_to_db<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Maybe saving UTXOs...");
    match &state.utxos_and_values.len() {
        0 => {
            info!("✔ No UTXOs in state to save!");
            Ok(state)
        },
        _ => save_utxos_to_db(state.db, &state.utxos_and_values).and(Ok(state)),
    }
}
