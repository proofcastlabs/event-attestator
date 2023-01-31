use ethereum_types::U256;

use crate::{
    chains::eth::eth_state::EthState,
    erc20_on_int::int::eth_tx_info::{Erc20OnIntEthTxInfo, Erc20OnIntEthTxInfos},
    traits::DatabaseInterface,
    types::Result,
};

impl Erc20OnIntEthTxInfos {
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
                .collect::<Vec<Erc20OnIntEthTxInfo>>(),
        ))
    }
}

pub fn filter_out_zero_value_eth_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe filtering out zero value `Erc20OnIntEthTxInfos`...");
    debug!(
        "✔ Num `Erc20OnIntEthTxInfos` before: {}",
        state.erc20_on_int_eth_signed_txs.len()
    );
    state
        .erc20_on_int_eth_tx_infos
        .filter_out_zero_values()
        .and_then(|filtered_tx_infos| {
            debug!("✔ Num `Erc20OnIntEthTxInfos` after: {}", filtered_tx_infos.len());
            state.replace_erc20_on_int_eth_tx_infos(filtered_tx_infos)
        })
}
