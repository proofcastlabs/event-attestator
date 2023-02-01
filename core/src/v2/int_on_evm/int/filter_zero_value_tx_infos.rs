use ethereum_types::U256;

use crate::{
    chains::eth::eth_state::EthState,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    int_on_evm::int::evm_tx_info::{IntOnEvmEvmTxInfo, IntOnEvmEvmTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

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
    info!("✔ Maybe filtering out zero value `IntOnEvmEvmTxInfos`...");
    debug!(
        "✔ Num `IntOnEvmEvmTxInfos` before: {}",
        state.int_on_evm_int_tx_infos.len()
    );
    state
        .int_on_evm_evm_tx_infos
        .filter_out_zero_values(&EthEvmTokenDictionary::get_from_db(state.db)?)
        .and_then(|filtered_tx_infos| {
            debug!("✔ Num `IntOnEvmEvmTxInfos` after: {}", filtered_tx_infos.len());
            state.replace_int_on_evm_evm_tx_infos(filtered_tx_infos)
        })
}
