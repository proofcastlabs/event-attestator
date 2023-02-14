use common::{traits::DatabaseInterface, types::Result};
use common_btc::{BtcState, MINIMUM_REQUIRED_SATOSHIS};

use crate::{
    btc::eth_tx_info::{BtcOnEthEthTxInfo, BtcOnEthEthTxInfos},
    utils::convert_satoshis_to_wei,
};

impl BtcOnEthEthTxInfos {
    pub fn filter_out_value_too_low(&self) -> Result<Self> {
        info!(
            "✔ Filtering out any eth tx infos below a minimum of {} Satoshis...",
            MINIMUM_REQUIRED_SATOSHIS
        );
        let threshold = convert_satoshis_to_wei(MINIMUM_REQUIRED_SATOSHIS);
        Ok(BtcOnEthEthTxInfos::new(
            self.iter()
                .filter(|params| match params.amount >= threshold {
                    true => true,
                    false => {
                        info!("✘ Filtering eth tx infos ∵ value too low: {:?}", params);
                        false
                    },
                })
                .cloned()
                .collect::<Vec<BtcOnEthEthTxInfo>>(),
        ))
    }
}

pub fn maybe_filter_out_value_too_low_btc_on_eth_eth_tx_infos_in_state<D: DatabaseInterface>(
    state: BtcState<D>,
) -> Result<BtcState<D>> {
    if state.tx_infos.is_empty() {
        warn!("✘ Not filtering value too low tx infos because there are none to filter!");
        Ok(state)
    } else {
        BtcOnEthEthTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| infos.filter_out_value_too_low())
            .and_then(|params| params.to_bytes())
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_eth_tx_infos;

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
