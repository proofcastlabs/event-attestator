use common::{
    chains::eth::EthState,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    traits::DatabaseInterface,
    types::Result,
};
use ethereum_types::U256;

use crate::int::evm_tx_info::{IntOnEvmEvmTxInfo, IntOnEvmEvmTxInfos};

impl IntOnEvmEvmTxInfos {
    fn get_host_token_amounts(&self, dictionary: &EthEvmTokenDictionary) -> Result<Vec<U256>> {
        self.iter()
            .map(|tx_info| tx_info.get_host_token_amount(dictionary))
            .collect::<Result<Vec<U256>>>()
    }

    pub fn filter_out_zero_values(&self, dictionary: &EthEvmTokenDictionary) -> Result<Self> {
        let host_token_amounts = self.get_host_token_amounts(dictionary)?;
        Ok(Self::new(
            self.iter()
                .zip(host_token_amounts.iter())
                .filter(|(tx_info, evm_amount)| match *evm_amount != &U256::zero() {
                    true => true,
                    false => {
                        info!(
                            "✘ Filtering out peg in info due to zero INT asset amount: {:?}",
                            tx_info
                        );
                        false
                    },
                })
                .map(|(info, _)| info)
                .cloned()
                .collect::<Vec<IntOnEvmEvmTxInfo>>(),
        ))
    }
}

pub fn filter_out_zero_value_evm_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        warn!("✘ Not filtering out zero value `IntOnEvmEvmTxInfos`, none to filter");
        Ok(state)
    } else {
        let dictionary = EthEvmTokenDictionary::get_from_db(state.db)?;
        IntOnEvmEvmTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                debug!("✔ Num `IntOnEvmEvmTxInfos` before: {}", tx_infos.len());
                tx_infos.filter_out_zero_values(&dictionary)
            })
            .and_then(|filtered_tx_infos| {
                debug!("✔ Num `IntOnEvmEvmTxInfos` after: {}", filtered_tx_infos.len());
                filtered_tx_infos.to_bytes()
            })
            .map(|bytes| state.add_tx_infos(bytes))
    }
}
