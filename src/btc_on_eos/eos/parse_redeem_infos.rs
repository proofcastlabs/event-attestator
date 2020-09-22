use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::eos_state::EosState,
    chains::eos::parse_redeem_infos::parse_redeem_infos_from_action_proofs,
};

pub fn maybe_parse_redeem_infos_and_put_in_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Parsing redeem params from actions data...");
    parse_redeem_infos_from_action_proofs(&state.action_proofs)
        .and_then(|params| {
            info!("✔ Parsed {} sets of params!", params.len());
            state.add_redeem_infos(params)
        })
}
