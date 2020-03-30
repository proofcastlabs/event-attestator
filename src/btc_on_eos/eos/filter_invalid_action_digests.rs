use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::ActionProofs,
    },
};

fn filter_out_invalid_action_receipt_digests(
    action_proofs: &ActionProofs
) -> Result<ActionProofs> {
    Ok(
        action_proofs
            .iter()
            .map(|proof| proof.action_receipt.to_digest())
            .map(hex::encode)
            .zip(action_proofs.iter())
            .filter(|(digest, proof)| digest == &proof.action_proof[0])
            .map(|(_, proof)| proof)
            .cloned()
            .collect::<ActionProofs>()
    )
}

pub fn maybe_filter_out_invalid_action_receipt_digests<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out invalid action digests...");
    filter_out_invalid_action_receipt_digests(&state.action_proofs)
        .and_then(|proofs| state.replace_action_proofs(proofs))
}

#[cfg(test)]
 mod tests {
     use super::*;
     use crate::btc_on_eos::{
         eos::eos_test_utils::get_sample_eos_submission_material_n,
     };

     #[test]
     fn should_not_filter_out_valid_action_receipt_digests() {
         let action_proofs = vec![
             // TODO Make an eos_test_utils helper to get `action_proof_n(...)`
             get_sample_eos_submission_material_n(4)
                 .action_proofs[0]
                 .clone(),
             get_sample_eos_submission_material_n(5)
                 .action_proofs[0]
                 .clone(),
         ];
         let expected_num_results = action_proofs.len();
         let result = filter_out_invalid_action_receipt_digests(&action_proofs)
             .unwrap();
         assert_eq!(result.len(), expected_num_results);
     }

     #[test]
     fn should_filter_out_invalid_action_receipt_digests() {
         // TODO Change some data in the action_receipt to invalidate the digest
         // &/| change the first element of the action proof itself.
         assert!(false);
     }
 }

