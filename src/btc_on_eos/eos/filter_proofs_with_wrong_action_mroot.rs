use eos_primitives::Checksum256;
use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::ActionProofs,
    },
};

fn filter_proofs_with_wrong_action_mroot(
    action_mroot: &Checksum256,
    action_proofs: &ActionProofs,
) -> Result<ActionProofs> {
    Ok(
        action_proofs
            .iter()
            .filter(|proof_data|
                proof_data.action_proof[proof_data.action_proof.len() - 1] ==
                action_mroot.to_string()
            )
            .cloned()
            .collect::<ActionProofs>()
    )
}

pub fn maybe_filter_out_proofs_with_wrong_action_mroot<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out proofs with wrong `action_mroot`...");
    filter_proofs_with_wrong_action_mroot(
        &state.get_eos_block_header()?.action_mroot,
        &state.action_proofs,
    )
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
