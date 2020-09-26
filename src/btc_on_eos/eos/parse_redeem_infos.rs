use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        parse_redeem_infos::parse_redeem_infos_from_action_proofs,
    },
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
