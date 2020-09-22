use crate::{
    types::Result,
    chains::{
        btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS,
        eos::eos_types::{
            RedeemInfo,
            RedeemInfos,
        },
    },
};

pub fn filter_redeem_infos(redeem_infos: &RedeemInfos) -> Result<RedeemInfos> {
    Ok(
        RedeemInfos::new(
            redeem_infos
                .0
                .iter()
                .map(|redeem_info| redeem_info.amount)
                .zip(redeem_infos.0.iter())
                .filter(|(amount, redeem_info)| {
                    match amount >= &MINIMUM_REQUIRED_SATOSHIS {
                        true => true,
                        false => {
                            info!("✘ Filtering redeem redeem info ∵ value too low: {:?}", redeem_info);
                            false
                        }
                    }
                })
                .map(|(_, redeem_info)| redeem_info)
                .cloned()
                .collect::<Vec<RedeemInfo>>()
        )
    )
}
