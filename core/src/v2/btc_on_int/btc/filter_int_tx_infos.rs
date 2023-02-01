use crate::{
    chains::btc::{btc_constants::MINIMUM_REQUIRED_SATOSHIS, btc_utils::convert_satoshis_to_wei},
    state::BtcState,
    traits::DatabaseInterface,
    tx_infos::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos},
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
                .filter(|params| match params.host_token_amount >= threshold {
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

// TODO Test!
