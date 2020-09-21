use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::eos_state::EosState,
    chains::eos::filter_action_proofs::filter_out_proofs_with_invalid_merkle_proofs,
};

pub fn maybe_filter_out_proofs_with_invalid_merkle_proofs<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out invalid merkle proofs...");
    filter_out_proofs_with_invalid_merkle_proofs(&state.action_proofs)
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
