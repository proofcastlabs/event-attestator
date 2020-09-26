use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        filter_redeem_infos::filter_redeem_infos,
    },
};

pub fn maybe_filter_value_too_low_redeem_infos_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out any redeem infos below minimum # of Satoshis...");
    filter_redeem_infos(&state.redeem_infos).and_then(|new_infos| state.replace_redeem_infos(new_infos))
}
