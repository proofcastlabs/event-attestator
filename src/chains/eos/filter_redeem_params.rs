use crate::{
    types::Result,
    chains::{
        eos::eos_types::RedeemInfo,
        btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS,
    },
};

pub fn filter_redeem_params(redeem_params: &[RedeemInfo]) -> Result<Vec<RedeemInfo>> {
    Ok(
        redeem_params
            .iter()
            .map(|params| params.amount)
            .zip(redeem_params.iter())
            .filter(|(amount, params)| {
                match amount >= &MINIMUM_REQUIRED_SATOSHIS {
                    true => true,
                    false => {
                        info!("✘ Filtering redeem params ∵ value too low: {:?}", params);
                        false
                    }
                }
            })
            .map(|(_, params)| params)
            .cloned()
            .collect::<Vec<RedeemInfo>>()
    )
}
