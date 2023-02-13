use common::{traits::DatabaseInterface, types::Result};
use common_eos::{EosState, ProcessedGlobalSequences};

use crate::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos};

impl IntOnEosIntTxInfos {
    pub fn filter_out_already_processed_txs(&self, processed_tx_ids: &ProcessedGlobalSequences) -> Result<Self> {
        Ok(IntOnEosIntTxInfos::new(
            self.iter()
                .filter(|info| !processed_tx_ids.contains(&info.global_sequence))
                .cloned()
                .collect::<Vec<IntOnEosIntTxInfo>>(),
        ))
    }
}

pub fn maybe_filter_out_already_processed_tx_infos_from_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    if state.tx_infos.is_empty() {
        warn!("✘ Not filtering out already-processed tx infos since there none to filter!");
        Ok(state)
    } else {
        info!("✔ Filtering out already processed tx infos...");
        IntOnEosIntTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                debug!("✔ Num before filtering: {}", tx_infos.len());
                tx_infos.filter_out_already_processed_txs(&state.processed_tx_ids)
            })
            .and_then(|filtered| {
                debug!("✔ Num after filtering: {}", filtered.len());
                filtered.to_bytes()
            })
            .map(|bytes| state.add_tx_infos(bytes))
    }
}
