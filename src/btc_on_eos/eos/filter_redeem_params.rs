use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::eos_state::EosState,
    chains::eos::filter_redeem_params::filter_redeem_params,
};

pub fn maybe_filter_value_too_low_redeem_params_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out any redeem params below minimum # of Satoshis...");
    filter_redeem_params(&state.redeem_params).and_then(|new_params| state.replace_redeem_params(new_params))
}
