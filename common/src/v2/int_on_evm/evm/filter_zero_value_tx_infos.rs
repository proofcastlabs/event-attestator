use ethereum_types::U256;

use crate::{
    int_on_evm::evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnEvmIntTxInfos {
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
                .collect::<Vec<IntOnEvmIntTxInfo>>(),
        ))
    }
}

pub fn filter_out_zero_value_eth_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        warn!("✘ Not filtering out zero value `IntOnEvmIntTxInfos`, none to filter");
        Ok(state)
    } else {
        IntOnEvmIntTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                debug!("✔ Num `IntOnEvmIntTxInfos` before: {}", tx_infos.len());
                tx_infos.filter_out_zero_values()
            })
            .and_then(|filtered_tx_infos| {
                debug!("✔ Num `IntOnEvmIntTxInfos` after: {}", filtered_tx_infos.len());
                filtered_tx_infos.to_bytes()
            })
            .map(|bytes| state.add_tx_infos(bytes))
    }
}
