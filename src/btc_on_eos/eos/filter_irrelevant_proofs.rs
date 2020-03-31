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
    use crate::btc_on_eos::eos::eos_test_utils::get_sample_action_proof_n;

    #[test]
    fn should_not_filter_out_proofs_for_valid_account() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(5),
        ];
        let account = EosAccountName::from_str("pbtctokenxxx").unwrap();

        assert_eq!(filter_out_proofs_for_other_accounts(&action_proofs, &account).unwrap(), action_proofs);
    }

    #[test]
    fn should_filter_out_proofs_for_other_accounts() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(5),
        ];
        let account = EosAccountName::from_str("provtestable").unwrap();

        assert_eq!(filter_out_proofs_for_other_accounts(&action_proofs, &account).unwrap(), []);
    }

    #[test]
    fn should_not_filter_out_proofs_for_valid_actions() {
        let action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(5),
        ];
        let result = filter_out_proofs_for_other_actions(&action_proofs)
            .unwrap();

        assert_eq!(result, action_proofs);
    }

    #[test]
    fn should_filter_out_proofs_for_other_actions() {
        let valid_action_name = EosActionName::from_str(REDEEM_ACTION_NAME).unwrap();
        let invalid_action_name = EosActionName::from_str("setproducers").unwrap();

        let mut dirty_action_proofs = vec![
            get_sample_action_proof_n(4),
            get_sample_action_proof_n(5),
        ];
        dirty_action_proofs[0].action.name = invalid_action_name;

        assert_ne!(dirty_action_proofs[0].action.name, valid_action_name);

        let result = filter_out_proofs_for_other_actions(&dirty_action_proofs)
            .unwrap();

        assert_eq!(result, [get_sample_action_proof_n(5)]);
    }
}
