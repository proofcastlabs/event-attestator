use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::eos_state::EosState,
    chains::{
        eos::eos_types::RedeemParams,
        btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS,
    },
};

fn filter_redeem_params(
    redeem_params: &[RedeemInfo],
) -> Result<Vec<RedeemInfo>> {
    Ok(
        redeem_params
            .iter()
            .map(|params| params.amount)
            .zip(redeem_params.iter())
            .filter(|(amount, params)| {
                match amount >= &MINIMUM_REQUIRED_SATOSHIS {
                    true => true,
                    false => {
                        info!(
                            "✘ Filtering redeem params ∵ value too low: {:?}",
                            params,
                        );
                        false
                    }
                }
            })
            .map(|(_, params)| params)
            .cloned()
            .collect::<Vec<RedeemInfo>>()
    )
}

pub fn maybe_filter_value_too_low_redeem_params_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out any redeem params below minimum # of Satoshis...");
    filter_redeem_params(&state.redeem_params)
        .and_then(|new_params| state.replace_redeem_params(new_params))
}
