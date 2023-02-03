use common::{dictionaries::eth_evm::EthEvmTokenDictionary, state::EthState, traits::DatabaseInterface, types::Result};
use ethereum_types::U256;

use crate::eth::int_tx_info::{Erc20OnIntIntTxInfo, Erc20OnIntIntTxInfos};

impl Erc20OnIntIntTxInfo {
    pub fn get_host_token_amount(&self, dictionary: &EthEvmTokenDictionary) -> Result<U256> {
        dictionary.convert_eth_amount_to_evm_amount(&self.eth_token_address, self.native_token_amount)
    }
}

impl Erc20OnIntIntTxInfos {
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
                .collect::<Vec<Erc20OnIntIntTxInfo>>(),
        ))
    }
}

pub fn filter_out_zero_value_evm_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe filtering out zero value `Erc20OnIntIntTxInfos`...");
    debug!(
        "✔ Num `Erc20OnIntIntTxInfos` before: {}",
        state.erc20_on_int_int_signed_txs.len()
    );
    Erc20OnIntIntTxInfos::from_bytes(&state.tx_infos)
        .and_then(|tx_infos| tx_infos.filter_out_zero_values(&EthEvmTokenDictionary::get_from_db(state.db)?))
        .and_then(|filtered_tx_infos| {
            debug!("✔ Num `Erc20OnIntIntTxInfos` after: {}", filtered_tx_infos.len());
            filtered_tx_infos.to_bytes()
        })
        .map(|bytes| state.add_tx_infos(bytes))
}
