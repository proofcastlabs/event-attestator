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
    Ok(RedeemInfos::new(
        &redeem_infos
            .0
            .iter()
            .map(|redeem_info| redeem_info.amount)
            .zip(redeem_infos.0.iter())
            .filter_map(|(amount, redeem_info)| {
                match amount >= MINIMUM_REQUIRED_SATOSHIS {
                    true => Some(redeem_info),
                    false => {
                        info!("✘ Filtering redeem redeem info ∵ value too low: {:?}", redeem_info);
                        None
                    }
                }
            })
            .cloned()
            .collect::<Vec<RedeemInfo>>()
    ))
}
