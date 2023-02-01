use ethereum_types::U256;

use crate::{
    dictionaries::eos_eth::EosEthTokenDictionary,
    eos_on_int::{
        constants::MINIMUM_WEI_AMOUNT,
        int::eos_tx_info::{EosOnIntEosTxInfo, EosOnIntEosTxInfos},
    },
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

impl EosOnIntEosTxInfos {
    pub fn filter_out_those_with_zero_eos_asset_amount(&self, dictionary: &EosEthTokenDictionary) -> Self {
        info!("✔ Filtering out `EosOnIntEosTxInfos` if they have a zero EOS asset amount...");
        Self::new(
            self.iter()
                .filter(|tx_info| {
                    match dictionary.get_zero_eos_asset_amount_via_eth_token_address(&tx_info.int_token_address) {
                        Err(_) => {
                            info!(
                                "✘ Filtering out tx ∵ cannot determine zero EOS asset amount! {:?}",
                                tx_info
                            );
                            false
                        },
                        Ok(zero_asset_amount) => tx_info.eos_asset_amount != zero_asset_amount,
                    }
                })
                .cloned()
                .collect::<Vec<EosOnIntEosTxInfo>>(),
        )
    }

    pub fn filter_out_those_with_value_too_low(&self) -> Result<Self> {
        let min_amount = U256::from_dec_str(MINIMUM_WEI_AMOUNT)?;
        Ok(EosOnIntEosTxInfos::new(
            self.iter()
                .filter(|info| {
                    if info.token_amount >= min_amount {
                        true
                    } else {
                        info!("✘ Filtering out tx info ∵ value too low: {:?}", info);
                        false
                    }
                })
                .cloned()
                .collect::<Vec<EosOnIntEosTxInfo>>(),
        ))
    }
}

pub fn maybe_filter_out_int_tx_info_with_value_too_low_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe filtering `EosOnIntEosTxInfos`...");
    let tx_infos = state.eos_on_int_eos_tx_infos.clone();
    debug!("✔ Num tx infos before: {}", tx_infos.len());
    tx_infos
        .filter_out_those_with_value_too_low()
        .and_then(|filtered_infos| {
            debug!("✔ Num tx infos after: {}", filtered_infos.len());
            state.replace_eos_on_int_eos_tx_infos(filtered_infos)
        })
}

pub fn maybe_filter_out_zero_eos_asset_amounts_in_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    info!("✔ Maybe filtering out zero eos asset amounts in state...");
    let dictionary = EosEthTokenDictionary::get_from_db(state.db)?;
    let filtered = state
        .eos_on_int_eos_tx_infos
        .filter_out_those_with_zero_eos_asset_amount(&dictionary);
    state.replace_eos_on_int_eos_tx_infos(filtered)
}
