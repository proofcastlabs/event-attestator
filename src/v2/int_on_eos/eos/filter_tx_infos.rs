use crate::{
    chains::eos::{eos_global_sequences::ProcessedGlobalSequences, eos_state::EosState},
    int_on_eos::eos::int_tx_info::{IntOnEosIntTxInfo, IntOnEosIntTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

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
    info!("✔ Filtering out already processed tx infos...");
    state
        .int_on_eos_int_tx_infos
        .filter_out_already_processed_txs(&state.processed_tx_ids)
        .and_then(|filtered| state.replace_int_on_eos_int_tx_infos(filtered))
}
