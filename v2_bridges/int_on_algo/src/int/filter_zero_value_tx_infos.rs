use common::{traits::DatabaseInterface, types::Result};
use common_eth::EthState;
use ethereum_types::U256;

use crate::int::algo_tx_info::{IntOnAlgoAlgoTxInfo, IntOnAlgoAlgoTxInfos};

impl IntOnAlgoAlgoTxInfos {
    fn get_host_token_amounts(&self) -> Vec<U256> {
        self.iter()
            .map(|tx_info| tx_info.host_token_amount)
            .collect::<Vec<U256>>()
    }

    pub fn filter_out_zero_values(&self) -> Result<Self> {
        let host_token_amounts = self.get_host_token_amounts();
        Ok(Self::new(
            self.iter()
                .zip(host_token_amounts.iter())
                .filter(
                    |(tx_info, host_token_amount)| match *host_token_amount != &U256::zero() {
                        true => true,
                        false => {
                            info!(
                                "✘ Filtering out peg in info due to zero ALGO asset amount: {:?}",
                                tx_info
                            );
                            false
                        },
                    },
                )
                .map(|(info, _)| info)
                .cloned()
                .collect::<Vec<IntOnAlgoAlgoTxInfo>>(),
        ))
    }
}

pub fn filter_out_zero_value_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ Not filtering out zero value tx infos because there aren't any in state!");
        Ok(state)
    } else {
        info!("✔ Filtering out zero value `IntOnAlgoAlgoTxInfos`...");
        IntOnAlgoAlgoTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| {
                debug!("✔ Num `IntOnAlgoAlgoTxInfos` before: {}", infos.len());
                infos.filter_out_zero_values()
            })
            .and_then(|filtered_tx_infos| {
                debug!("✔ Num `IntOnAlgoAlgoTxInfos` after: {}", filtered_tx_infos.len());
                filtered_tx_infos.to_bytes()
            })
            .map(|bytes| state.add_tx_infos(bytes))
    }
}
