use common::{traits::DatabaseInterface, types::Result};
use common_chain_ids::EosChainId;
use common_eos::{
    filter_for_proofs_with_action_name,
    EosState,
    REDEEM_ACTION_NAME as V1_REDEEM_ACTION_NAME,
    V2_REDEEM_ACTION_NAME,
};

pub fn maybe_filter_for_relevant_redeem_actions<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    // NOTE: We care about ALL `redeem2` actions...
    info!("✔ Filtering for `redeem2` actions...");
    let v2_redeem_proofs = filter_for_proofs_with_action_name(&state.action_proofs, V2_REDEEM_ACTION_NAME)?;
    debug!("found {} v2 redeem proofs", v2_redeem_proofs.len());

    let v1_redeem_proofs = if matches!(
        state.eos_db_utils.get_eos_chain_id_from_db(),
        Ok(EosChainId::UltraMainnet)
    ) {
        // NOTE: However we also occasionally want some v1 `redeem` actions, for custom handling,
        // in order to facilitate v1->v2 bridge migrations in cases where it's not so easy on chain.
        let proofs = filter_for_proofs_with_action_name(&state.action_proofs, V1_REDEEM_ACTION_NAME)?;
        debug!("found {} v1 redeem proofs for ultra mainnet", proofs.len());
        proofs
    } else {
        vec![]
    };

    state.replace_action_proofs([v1_redeem_proofs, v2_redeem_proofs].concat())
}
