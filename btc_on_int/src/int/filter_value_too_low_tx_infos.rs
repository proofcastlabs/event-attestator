#[cfg(not(test))]
use common::chains::btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS;

use crate::int::{BtcOnIntBtcTxInfo, BtcOnIntBtcTxInfos};

impl BtcOnIntBtcTxInfos {
    pub fn filter_out_any_whose_value_is_too_low(&self) -> Self {
        info!("✘ Filtering out `BtcOnIntBtcTxInfo` whose amounts are too low...");
        Self::new(
            self.iter()
                .filter(|redeem_info| {
                    #[cfg(not(test))]
                    let amount_is_too_low = redeem_info.amount_in_satoshis < MINIMUM_REQUIRED_SATOSHIS;
                    #[cfg(test)]
                    let amount_is_too_low = redeem_info.amount_in_satoshis < 100;
                    if amount_is_too_low {
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
