use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        filter_action_proofs::filter_out_proofs_with_action_digests_not_in_action_receipts,
    },
};

pub fn maybe_filter_out_action_proof_receipt_mismatches_and_return_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering proofs w/ action digests NOT in action receipts...");
    filter_out_proofs_with_action_digests_not_in_action_receipts(&state.action_proofs)
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
