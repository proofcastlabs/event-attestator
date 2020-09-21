use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::eos::eos_state::EosState,
    chains::eos::filter_action_proofs::filter_out_invalid_action_receipt_digests,
};

pub fn maybe_filter_out_invalid_action_receipt_digests<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out invalid action digests...");
    filter_out_invalid_action_receipt_digests(&state.action_proofs)
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
