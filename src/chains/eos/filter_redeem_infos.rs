use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::redeem_info::{
        BtcOnEosRedeemInfo,
        BtcOnEosRedeemInfos,
    },
    chains::{
        eos::eos_state::EosState,
        btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS,
    },
};

pub fn filter_btc_on_eos_redeem_infos(redeem_infos: &BtcOnEosRedeemInfos) -> Result<BtcOnEosRedeemInfos> {
    Ok(BtcOnEosRedeemInfos::new(
        redeem_infos
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
            .collect::<Vec<BtcOnEosRedeemInfo>>()
    ))
}

pub fn maybe_filter_value_too_low_redeem_infos_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out any redeem infos below minimum # of Satoshis...");
    filter_btc_on_eos_redeem_infos(&state.btc_on_eos_redeem_infos)
        .and_then(|new_infos| state.replace_btc_on_eos_redeem_infos(new_infos))
}
