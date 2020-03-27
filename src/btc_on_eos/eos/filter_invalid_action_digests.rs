use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::ActionProofs,
    },
};

fn filter_out_invalid_action_digests(
    action_proofs: &ActionProofs
) -> Result<ActionProofs> {
    Ok(
        action_proofs
            .iter()
            .map(|proof| proof.action.to_digest())
            .zip(action_proofs.iter())
            .filter(|(digest, proof)|
                hex::encode(digest) ==
                proof.action_proof[proof.action_proof.len() - 1]
            )
            .map(|(_, proof)| proof)
            .cloned()
            .collect::<ActionProofs>()
    )
}

pub fn maybe_filter_out_invalid_action_digests<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out invalid action digests...");
    filter_out_invalid_action_digests(&state.action_proofs)
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
