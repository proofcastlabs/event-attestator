use eos_primitives::Checksum256;
use crate::btc_on_eos::{
    traits::DatabaseInterface,
    utils::convert_bytes_to_checksum256,
    types::{
        Bytes,
        Result,
    },
    eos::{
        eos_state::EosState,
        eos_types::ActionProofs,
    },
};

fn filter_out_proofs_with_action_digests_not_in_action_receipts(
    action_proofs: &ActionProofs
) -> Result<ActionProofs> {
    Ok(
        action_proofs
            .iter()
            .map(|proof| proof.action.to_digest())
            .map(|digest_bytes| convert_bytes_to_checksum256(&digest_bytes))
            .collect::<Result<Vec<Checksum256>>>()?
            .into_iter()
            .zip(action_proofs.iter())
            .filter(|(digest, proof)| digest == &proof.action_receipt.act_digest)
            .map(|(_, proof)| proof)
            .cloned()
            .collect::<ActionProofs>()
    )
}

pub fn maybe_filter_out_action_proof_receipt_mismatches<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering proofs w/ action digests NOT in action receipts...");
    filter_out_proofs_with_action_digests_not_in_action_receipts(
        &state.action_proofs
    )
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
