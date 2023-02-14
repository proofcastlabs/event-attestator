use common::{traits::DatabaseInterface, types::Result};
use common_btc::{convert_satoshis_to_wei, BtcState, MINIMUM_REQUIRED_SATOSHIS};

use crate::btc::{BtcOnIntIntTxInfo, BtcOnIntIntTxInfos};

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
    BtcOnIntIntTxInfos::from_bytes(&state.tx_infos)
        .and_then(|infos| infos.filter_out_value_too_low())
        .and_then(|filtered| filtered.to_bytes())
        .map(|bytes| state.add_tx_infos(bytes))
}
