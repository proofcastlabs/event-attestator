use common::{
    chains::eos::eos_global_sequences::ProcessedGlobalSequences,
    chains::eos::EosState,
    traits::DatabaseInterface,
    types::Result,
};
use ethereum_types::U256;

use crate::{
    constants::MINIMUM_WEI_AMOUNT,
    eos::int_tx_info::{EosOnIntIntTxInfo, EosOnIntIntTxInfos},
};

impl EosOnIntIntTxInfos {
    pub fn filter_out_already_processed_txs(&self, processed_tx_ids: &ProcessedGlobalSequences) -> Result<Self> {
        Ok(EosOnIntIntTxInfos::new(
            self.iter()
                .filter(|info| !processed_tx_ids.contains(&info.global_sequence))
                .cloned()
                .collect::<Vec<EosOnIntIntTxInfo>>(),
        ))
    }

    pub fn filter_out_those_with_value_too_low(&self) -> Result<Self> {
        let min_amount = U256::from_dec_str(MINIMUM_WEI_AMOUNT)?;
        Ok(EosOnIntIntTxInfos::new(
            self.iter()
                .filter(|info| {
                    if info.amount >= min_amount {
                        true
                    } else {
                        info!("✘ Filtering out tx info ∵ value too low: {:?}", info);
                        false
                    }
                })
                .cloned()
                .collect::<Vec<EosOnIntIntTxInfo>>(),
        ))
    }
}

pub fn maybe_filter_out_already_processed_tx_ids_from_state<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Filtering out already processed tx IDs...");
    EosOnIntIntTxInfos::from_bytes(&state.tx_infos)
        .and_then(|tx_infos| {
            debug!("Num tx infos before: {}", tx_infos.len());
            tx_infos.filter_out_already_processed_txs(&state.processed_tx_ids)
        })
        .and_then(|filtered| {
            debug!("Num tx infos after: {}", filtered.len());
            filtered.to_bytes()
        })
        .map(|bytes| state.add_tx_infos(bytes))
}

pub fn maybe_filter_out_value_too_low_txs_from_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Filtering out value too low txs from state...");
    EosOnIntIntTxInfos::from_bytes(&state.tx_infos)
        .and_then(|tx_infos| {
            debug!("Num tx infos before: {}", tx_infos.len());
            tx_infos.filter_out_those_with_value_too_low()
        })
        .and_then(|filtered| {
            debug!("Num tx infos after: {}", &filtered.len());
            filtered.to_bytes()
        })
        .map(|bytes| state.add_tx_infos(bytes))
}
