use std::str::FromStr;
use eos_primitives::{
    ActionName as EosActionName,
    AccountName as EosAccountName,
};
use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::ActionProofs,
        eos_constants::REDEEM_ACTION_NAME,
        eos_database_utils::get_eos_account_name_from_db,
    },
};

fn filter_out_proofs_for_other_accounts(
    action_proofs: &ActionProofs,
    required_account_name: &EosAccountName,
) -> Result<ActionProofs> {
    Ok(
        action_proofs
            .iter()
            .filter(|proof| &proof.action.account == required_account_name)
            .cloned()
            .collect()
    )
}

fn filter_out_proofs_for_other_actions(
    action_proofs: &ActionProofs
) -> Result<ActionProofs> {
    let required_action = EosActionName::from_str(REDEEM_ACTION_NAME)?;
    Ok(
        action_proofs
            .iter()
            .filter(|proof| proof.action.name == required_action)
            .cloned()
            .collect()
    )
}

// TODO Filter for those whose whose symbol isn't correct?
pub fn maybe_filter_out_irrelevant_proofs_from_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    info!("✔ Filtering out irrelevant proofs...");
    filter_out_proofs_for_other_accounts(
        &state.action_proofs,
        &get_eos_account_name_from_db(&state.db)?,
    )
        .and_then(|proofs| filter_out_proofs_for_other_actions(&proofs))
        .and_then(|proofs| state.replace_action_proofs(proofs))
}
