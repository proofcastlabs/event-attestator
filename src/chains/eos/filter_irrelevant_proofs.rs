use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::eos::{
        eos_state::EosState,
        eos_database_utils::get_eos_account_name_from_db,
        filter_action_proofs::{
            filter_out_proofs_for_other_actions,
            filter_out_proofs_for_other_accounts,
        }
    },
};

// TODO Filter for those whose symbol isn't correct?
pub fn maybe_filter_out_irrelevant_proofs_from_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out irrelevant proofs...");
    filter_out_proofs_for_other_accounts(&state.action_proofs, get_eos_account_name_from_db(&state.db)?)
        .and_then(|proofs| filter_out_proofs_for_other_actions(&proofs))
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
