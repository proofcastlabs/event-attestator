use common::{
    state::EthState,
    chains::{btc::utxo_manager::utxo_database_utils::save_utxos_to_db},
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_save_btc_utxos_to_db_and_return_state<D: DatabaseInterface>(
    state: EthState<D>
) -> Result<EthState<D>> {
    info!("✔ Maybe saving BTC UTXOs...");
    match &state.btc_utxos_and_values {
        Some(utxos) => save_utxos_to_db(state.db, utxos).and(Ok(state)),
        None => {
            info!("✔ No BTC UTXOs in state to save!");
            Ok(state)
        },
    }
}
