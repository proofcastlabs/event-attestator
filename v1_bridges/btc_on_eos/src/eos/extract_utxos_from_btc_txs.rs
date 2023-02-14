use common::{
    traits::{DatabaseInterface, Serdable},
    types::Result,
};
use common_btc::{extract_btc_utxo_from_btc_tx, BtcDbUtils};
use common_eos::EosState;

pub fn maybe_extract_btc_utxo_from_btc_tx_in_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Maybe extracting UTXOs from BTC txs in state...");
    match state.btc_on_eos_signed_txs.len() {
        0 => {
            info!("✔ No BTC txs in state ∴ no UTXOs to extract...");
            Ok(state)
        },
        _ => {
            info!("✔ Extracting BTC UTXOs...");
            extract_btc_utxo_from_btc_tx(&BtcDbUtils::new(state.db), &state.btc_on_eos_signed_txs)
                .and_then(|utxos| utxos.to_bytes())
                .map(|bytes| state.add_btc_utxos_and_values(bytes))
        },
    }
}
