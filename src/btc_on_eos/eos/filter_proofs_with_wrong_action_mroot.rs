use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        filter_action_proofs::filter_proofs_with_wrong_action_mroot,
    },
};

pub fn maybe_filter_out_proofs_with_wrong_action_mroot<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out proofs with wrong `action_mroot`...");
    filter_proofs_with_wrong_action_mroot(&state.get_eos_block_header()?.action_mroot, &state.action_proofs)
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
