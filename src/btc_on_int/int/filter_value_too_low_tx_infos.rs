use crate::{
    btc_on_int::int::btc_tx_info::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos},
    chains::btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS,
    types::Result,
};

impl BtcOnIntBtcTxInfos {
    pub fn filter_out_any_whose_value_is_too_low(&self) -> Self {
        info!("✘ Filtering out `BtcOnIntBtcTxInfo` whose amounts are too low...");
        Self::new(
            self.iter()
                .filter(|redeem_info| {
                    if redeem_info.amount_in_satoshis < MINIMUM_REQUIRED_SATOSHIS {
                        info!(
                            "✘ Filtering out `BtcOnIntBtcTxInfo` ∵ amount too low: {:?}",
                            redeem_info
                        );
                        false
                    } else {
                        true
                    }
                })
                .cloned()
                .collect::<Vec<BtcOnIntBtcTxInfo>>(),
        )
    }
}
