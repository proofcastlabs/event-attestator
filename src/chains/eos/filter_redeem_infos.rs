use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::{
        btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS,
        eos::{
            eos_state::EosState,
            eos_types::{
                BtcOnEthRedeemInfo,
                BtcOnEthRedeemInfos,
            },
        },
    },
};

pub fn filter_redeem_infos(redeem_infos: &BtcOnEthRedeemInfos) -> Result<BtcOnEthRedeemInfos> {
    Ok(BtcOnEthRedeemInfos::new(
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
            .collect::<Vec<BtcOnEthRedeemInfo>>()
    ))
}

pub fn maybe_filter_value_too_low_redeem_infos_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out any redeem infos below minimum # of Satoshis...");
    filter_redeem_infos(&state.redeem_infos).and_then(|new_infos| state.replace_redeem_infos(new_infos))
}
