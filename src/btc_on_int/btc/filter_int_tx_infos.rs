use crate::{
    btc_on_int::btc::int_tx_info::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos},
    chains::btc::{btc_constants::MINIMUM_REQUIRED_SATOSHIS, btc_state::BtcState, btc_utils::convert_satoshis_to_wei},
    traits::DatabaseInterface,
    types::Result,
};

impl BtcOnIntIntTxInfos {
    pub fn filter_out_value_too_low(&self) -> Result<Self> {
        info!(
            "✔ Filtering out any `BtcOnIntIntTxInfos` below a minimum of {} Satoshis...",
            MINIMUM_REQUIRED_SATOSHIS
        );
        let threshold = convert_satoshis_to_wei(MINIMUM_REQUIRED_SATOSHIS);
        Ok(BtcOnIntIntTxInfos::new(
            self.iter()
                .filter(|params| match params.amount >= threshold {
                    true => true,
                    false => {
                        info!("✘ Filtering eth tx infos ∵ value too low: {:?}", params);
                        false
                    },
                })
                .cloned()
                .collect::<Vec<BtcOnIntIntTxInfo>>(),
        ))
    }
}

pub fn maybe_filter_out_value_too_low_btc_on_int_int_tx_infos_in_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    state
        .btc_on_int_int_tx_infos
        .filter_out_value_too_low()
        .and_then(|filtered| state.replace_btc_on_int_int_tx_infos(filtered))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::btc::btc_test_utils::get_sample_eth_tx_infos;

    #[test]
    fn should_filter_eth_tx_infos() {
        let expected_length_before = 3;
        let expected_length_after = 2;
        let eth_tx_infos = get_sample_eth_tx_infos();
        let threshold = convert_satoshis_to_wei(MINIMUM_REQUIRED_SATOSHIS);
        let length_before = eth_tx_infos.len();
        assert_eq!(length_before, expected_length_before);
        let result = eth_tx_infos.filter_out_value_too_low().unwrap();
        let length_after = result.len();
        assert_eq!(length_after, expected_length_after);
        result.iter().for_each(|params| assert!(params.amount >= threshold));
    }
}
