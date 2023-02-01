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
    info!("✔ Maybe filtering out zero value `IntOnEvmIntTxInfos`...");
    debug!(
        "✔ Num `IntOnEvmIntTxInfos` before: {}",
        state.int_on_evm_int_tx_infos.len()
    );
    state
        .int_on_evm_int_tx_infos
        .filter_out_zero_values()
        .and_then(|filtered_tx_infos| {
            debug!("✔ Num `IntOnEvmIntTxInfos` after: {}", filtered_tx_infos.len());
            state.replace_int_on_evm_int_tx_infos(filtered_tx_infos)
        })
}
