use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        filter_action_proofs::filter_duplicate_proofs,
    },
};

pub fn maybe_filter_duplicate_proofs_from_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Maybe filtering duplicate proofs from state...");
    filter_duplicate_proofs(&state.action_proofs).and_then(|proofs| state.replace_action_proofs(proofs))
}
