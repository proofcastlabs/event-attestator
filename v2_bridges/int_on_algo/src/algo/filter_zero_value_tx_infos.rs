use common::{traits::DatabaseInterface, types::Result};
use common_algo::AlgoState;
use ethereum_types::U256;

use crate::algo::int_tx_info::IntOnAlgoIntTxInfos;

impl IntOnAlgoIntTxInfos {
    pub fn filter_out_zero_values(&self) -> Result<Self> {
        Ok(Self::new(
            self.iter()
                .filter(|tx_info| match tx_info.native_token_amount != U256::zero() {
                    true => true,
                    false => {
                        info!("✘ Filtering out redeem info due to zero asset amount: {:?}", tx_info);
                        false
                    },
                })
                .cloned()
                .collect(),
        ))
    }
}

pub fn filter_out_zero_value_tx_infos_from_state<D: DatabaseInterface>(state: AlgoState<D>) -> Result<AlgoState<D>> {
    if state.tx_infos.is_empty() {
        warn!("✔ Not filtering out zero value `IntOnAlgoIntTxInfos` because there aren't any!");
        Ok(state)
    } else {
        info!("✔ Filtering out zero value `IntOnAlgoIntTxInfos`...");
        IntOnAlgoIntTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                debug!("✔ Num `IntOnAlgoIntTxInfos` before: {}", tx_infos.len());
                tx_infos.filter_out_zero_values()
            })
            .and_then(|filtered_tx_infos| {
                debug!("✔ Num `IntOnAlgoIntTxInfos` after: {}", filtered_tx_infos.len());
                filtered_tx_infos.to_bytes()
            })
            .map(|bytes| state.add_tx_infos(bytes))
    }
}
