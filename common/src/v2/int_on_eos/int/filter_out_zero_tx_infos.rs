use std::str::FromStr;

use eos_chain::AccountName as EosAccountName;

use crate::{
    chains::eos::eos_utils::remove_symbol_from_eos_asset,
    dictionaries::eos_eth::EosEthTokenDictionary,
    int_on_eos::int::eos_tx_info::{IntOnEosEosTxInfo, IntOnEosEosTxInfos},
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};

impl IntOnEosEosTxInfo {
    pub fn is_zero_eos_amount(&self, dictionary: &EosEthTokenDictionary) -> Result<bool> {
        let entry = dictionary.get_entry_via_eos_address(&EosAccountName::from_str(&self.eos_token_address)?)?;
        let eos_amount = remove_symbol_from_eos_asset(&entry.convert_u256_to_eos_asset_string(&self.token_amount)?)
            .parse::<f64>()?;
        Ok(eos_amount == 0.0)
    }
}

impl IntOnEosEosTxInfos {
    pub fn filter_out_zero_eos_values(&self, dictionary: &EosEthTokenDictionary) -> Result<Self> {
        Ok(Self::new(
            self.iter()
                .map(|tx_info| {
                    let is_zero_eos_amount = tx_info.is_zero_eos_amount(dictionary)?;
                    Ok((tx_info.clone(), is_zero_eos_amount))
                })
                .collect::<Result<Vec<(IntOnEosEosTxInfo, bool)>>>()?
                .iter()
                .filter_map(|(tx_info, is_zero_amount)| {
                    if *is_zero_amount {
                        info!(
                            "✘ Filtering out peg in info due to zero EOS asset amount: {:?}",
                            tx_info
                        );
                        None
                    } else {
                        Some(tx_info)
                    }
                })
                .cloned()
                .collect::<Vec<IntOnEosEosTxInfo>>(),
        ))
    }
}

pub fn filter_out_zero_value_eos_tx_infos_from_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        warn!("✘ Not filtering tx infos for zero values since there are none!");
        Ok(state)
    } else {
        IntOnEosEosTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                debug!("✔ Num infos before: {}", tx_infos.len());
                tx_infos.filter_out_zero_eos_values(&EosEthTokenDictionary::get_from_db(state.db)?)
            })
            .and_then(|filtered_infos| {
                debug!("✔ Num infos after: {}", filtered_infos.len());
                filtered_infos.to_bytes()
            })
            .map(|bytes| state.add_tx_infos(bytes))
    }
}
