use crate::{
    chains::eos::{
        eos_constants::{REDEEM_ACTION_NAME, V2_REDEEM_ACTION_NAME},
        eos_state::EosState,
        filter_action_proofs::filter_for_proofs_with_action_name,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_filter_proofs_for_v1_and_v2_redeem_actions<D: DatabaseInterface>(
    state: EosState<D>,
) -> Result<EosState<D>> {
    info!("✔ Filtering out proofs with wrong `action_mroot`...");
    let v1_redeem_proofs = filter_for_proofs_with_action_name(&state.action_proofs, REDEEM_ACTION_NAME)?;
    debug!("found {} v1 redeem proofs", v1_redeem_proofs.len());
    let v2_redeem_proofs = filter_for_proofs_with_action_name(&state.action_proofs, V2_REDEEM_ACTION_NAME)?;
    debug!("found {} v2 redeem proofs", v2_redeem_proofs.len());
    state.replace_action_proofs([v1_redeem_proofs, v2_redeem_proofs].concat())
}
