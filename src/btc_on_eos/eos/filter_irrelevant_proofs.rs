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

// TODO Filter for those whose symbol isn't correct?
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::eos::eos_test_utils::get_sample_eos_submission_material_n;

    #[test]
    fn should_filter_out_proofs_for_other_accounts() {
        let action_data_1 = get_sample_eos_submission_material_n(4)
            .action_proofs[0]
            .clone();
        let action_data_2 = get_sample_eos_submission_material_n(5)
            .action_proofs[0]
            .clone();
        let action_proofs = vec![
            action_data_1.clone(),
            action_data_2.clone(),
        ];
        let account_1 = EosAccountName::from_str("provtestable").unwrap();
        let account_2 = EosAccountName::from_str("pbtctokenxxx").unwrap();

        assert_eq!(filter_out_proofs_for_other_accounts(&action_proofs, &account_1).unwrap(), []);
        assert_eq!(filter_out_proofs_for_other_accounts(&action_proofs, &account_2).unwrap(), action_proofs);
    }

    #[test]
    fn should_filter_out_proofs_for_other_actions() {
        let action_data_1 = get_sample_eos_submission_material_n(4)
            .action_proofs[0]
            .clone();
        let action_data_2 = get_sample_eos_submission_material_n(5)
            .action_proofs[0]
            .clone();
        let action_proofs_1 = vec![
            action_data_1.clone(),
        ];
        let action_proofs_2 = vec![
            action_data_1.clone(),
            action_data_2.clone(),
        ];

        assert_eq!(filter_out_proofs_for_other_actions(&action_proofs_1).unwrap(), action_proofs_1);
        assert_eq!(filter_out_proofs_for_other_actions(&action_proofs_2).unwrap(), action_proofs_2);
    }
}
