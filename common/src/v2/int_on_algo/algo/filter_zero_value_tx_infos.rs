use ethereum_types::U256;

use crate::{
    int_on_algo::algo::int_tx_info::IntOnAlgoIntTxInfos,
    state::AlgoState,
    traits::DatabaseInterface,
    types::Result,
};

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
    info!("✔ Maybe filtering out zero value `IntOnAlgoIntTxInfos`...");
    let tx_infos = state.get_int_on_algo_int_tx_infos();
    debug!("✔ Num `IntOnAlgoIntTxInfos` before: {}", tx_infos.len());
    tx_infos.filter_out_zero_values().and_then(|filtered_tx_infos| {
        debug!("✔ Num `IntOnAlgoIntTxInfos` after: {}", filtered_tx_infos.len());
        state.replace_int_on_algo_int_tx_infos(filtered_tx_infos)
    })
}

// TODO test!
