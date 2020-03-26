use eos_primitives::Checksum256;
use crate::btc_on_eos::{
    traits::DatabaseInterface,
    types::{
        Bytes,
        Result,
    },
    eos::{
        eos_state::EosState,
        eos_merkle_utils::get_merkle_digest,
        eos_types::{
            MerkleProof,
            ActionProofs,
        },
    },
};

fn get_merkle_digest_from_merkle_proof(
    merkle_proof: &MerkleProof
) -> String {
    hex::encode(
        get_merkle_digest(
            merkle_proof
                .iter()
                .map(|hex|
                    match hex::decode(hex) {
                        Ok(bytes) => bytes,
                        Err(_) => vec![0u8], // NOTE This make poof invalid
                    }
                )
                .collect::<Vec<Bytes>>()
        )
    )
}

fn filter_proofs_with_invalid_merkle_proofs(
    action_mroot: &Checksum256,
    action_proofs: &ActionProofs,
) -> Result<ActionProofs> {
    Ok(
        action_proofs
            .iter()
            .map(|proof_data| &proof_data.action_proof)
            .map(|proof| get_merkle_digest_from_merkle_proof(proof))
            .zip(action_proofs.iter())
            .filter(|(digest_hex, _)| digest_hex == &action_mroot.to_string())
            .map(|(_, proof)| proof)
            .cloned()
            .collect::<ActionProofs>()
    )
}

pub fn maybe_filter_out_proofs_with_wrong_merkle_roots<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out proofs w/ wrong merkle-roots...");
    filter_proofs_with_invalid_merkle_proofs(
        &state.get_eos_block_header()?.action_mroot,
        &state.action_proofs,
    )
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
