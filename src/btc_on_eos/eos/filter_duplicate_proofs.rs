use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    eos::{
        eos_state::EosState,
        eos_types::ActionsData,
    },
};

fn filter_duplicate_proofs(
    actions_data: &ActionsData
) -> Result<ActionsData> {
    let mut filtered: ActionsData = Vec::new();
    actions_data
        .iter()
        .map(|action_data| {
            if filtered.contains(&action_data) == false {
                filtered.push(action_data.clone())
            }
        })
        .for_each(drop);
    Ok(filtered)
}

pub fn maybe_filter_duplicate_proofs_from_state<D>(
    state: EosState<D>
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    filter_duplicate_proofs(&state.actions_data)
        .and_then(|filtered_proofs| state.replace_actions_data(filtered_proofs))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::{
        eos::eos_test_utils::get_sample_eos_submission_material_n,
    };

    #[test]
    fn should_not_filter_duplicate_action_proofs_if_there_are_no_duplicates() {
        let expected_num_proofs_after = 2;
        let expected_num_proofs_before = 2;
        let action_data_1 = get_sample_eos_submission_material_n(4)
            .actions_data[0]
            .clone();
        let action_data_2 = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .clone();
        let proofs_no_duplicates = vec![
            action_data_1.clone(),
            action_data_2.clone(),
        ];
        let num_proofs_before = proofs_no_duplicates.len();
        assert_eq!(num_proofs_before, expected_num_proofs_before);
        let result = filter_duplicate_proofs(&proofs_no_duplicates)
            .unwrap();
        assert_eq!(result.len(), num_proofs_before);
        assert_eq!(result.len(), expected_num_proofs_after);
        assert_eq!(result[0], action_data_1);
        assert_eq!(result[1], action_data_2);
    }
    #[test]
    fn should_filter_duplicate_action_proofs() {
        let expected_num_proofs_after = 2;
        let expected_num_proofs_before = 3;
        let action_data_1 = get_sample_eos_submission_material_n(4)
            .actions_data[0]
            .clone();
        let action_data_2 = get_sample_eos_submission_material_n(5)
            .actions_data[0]
            .clone();
        let proofs_with_duplicate = vec![
            action_data_1.clone(),
            action_data_2.clone(),
            action_data_2.clone(),
        ];
        let num_proofs_before = proofs_with_duplicate.len();
        assert_eq!(num_proofs_before, expected_num_proofs_before);
        let result = filter_duplicate_proofs(&proofs_with_duplicate)
            .unwrap();
        assert!(result.len() < num_proofs_before);
        assert_eq!(result.len(), expected_num_proofs_after);
        assert_eq!(result[0], action_data_1);
        assert_eq!(result[1], action_data_2);
    }
}
