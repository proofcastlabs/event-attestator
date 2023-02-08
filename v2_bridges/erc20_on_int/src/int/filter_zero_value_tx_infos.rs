use common::{chains::eth::EthState, traits::DatabaseInterface, types::Result};
use ethereum_types::U256;

use crate::int::eth_tx_info::{Erc20OnIntEthTxInfo, Erc20OnIntEthTxInfos};

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
    Erc20OnIntEthTxInfos::from_bytes(&state.tx_infos)
        .and_then(|tx_infos| tx_infos.filter_out_zero_values())
        .and_then(|filtered_tx_infos| {
            debug!("✔ Num `Erc20OnIntEthTxInfos` after: {}", filtered_tx_infos.len());
            filtered_tx_infos.to_bytes()
        })
        .map(|bytes| state.add_tx_infos(bytes))
}
