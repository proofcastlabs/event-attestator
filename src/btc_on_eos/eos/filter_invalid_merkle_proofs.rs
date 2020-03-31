use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::ActionProofs,
        eos_merkle_utils::verify_merkle_proof,
    },
};

fn filter_out_proofs_with_invalid_merkle_proofs(
    action_proofs: &ActionProofs,
) -> Result<ActionProofs> {
    Ok(
        action_proofs
            .iter()
            .map(|proof_data| &proof_data.action_proof)
            .map(verify_merkle_proof)
            .collect::<Result<Vec<bool>>>()?
            .into_iter()
            .zip(action_proofs.iter())
            .filter(|(proof_is_valid, _)| *proof_is_valid)
            .map(|(_, proof)| proof)
            .cloned()
            .collect::<ActionProofs>()
    )
}

pub fn maybe_filter_out_proofs_with_invalid_merkle_proofs<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out invalid merkle proofs...");
    filter_out_proofs_with_invalid_merkle_proofs(&state.action_proofs)
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
