use common::{traits::DatabaseInterface, types::Result};
use rust_algorand::{AlgorandBlock, AlgorandTransaction, AlgorandTransactions};

use crate::AlgoState;

fn remove_irrelevant_txs_from_block(block: &AlgorandBlock, asset_ids: Vec<u64>) -> Result<AlgorandTransactions> {
    Ok(block.get_transactions().map(|txs| {
        AlgorandTransactions::new(
            asset_ids
                .iter()
                .map(|id| txs.filter_by_transfer_asset_id(*id).0)
                .collect::<Vec<Vec<AlgorandTransaction>>>()
                .concat(),
        )
    })?)
}

pub fn remove_irrelevant_txs_from_submission_material_in_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    info!("✔ Removing irrelevant txs from submission material in state...");
    let mut submission_material = state.get_algo_submission_material()?;
    remove_irrelevant_txs_from_block(
        &submission_material.block,
        state.get_evm_algo_token_dictionary()?.to_algo_asset_ids(),
    )
    .and_then(|relevant_txs| {
        submission_material.block.transactions = if relevant_txs.is_empty() {
            None
        } else {
            Some(relevant_txs)
        };
        state.update_algo_submission_material(&submission_material)
    })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::{dictionaries::evm_algo::EvmAlgoTokenDictionary, test_utils::get_test_database};
    use serde_json::json;

    use super::*;
    use crate::test_utils::get_sample_submission_material_n;

    #[test]
    fn should_remove_all_txs_from_submission_material_in_state_if_none_are_relevant() {
        let dict = EvmAlgoTokenDictionary::from_str(
            &json!([{
                "evm_decimals": 18,
                "algo_decimals": 18,
                "algo_asset_id": 666,
                "evm_address": "0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8",
                "evm_symbol": "EVM",
                "algo_symbol": "ALGO",
            }])
            .to_string(),
        )
        .unwrap();
        let db = get_test_database();
        let submission_material = get_sample_submission_material_n(0);
        let state_1 = AlgoState::init(&db);
        let state_2 = state_1.add_evm_algo_dictionary(dict).unwrap();
        let state_3 = state_2.add_algo_submission_material(&submission_material).unwrap();
        assert!(
            state_3
                .get_algo_submission_material()
                .unwrap()
                .block
                .transactions
                .unwrap()
                .len()
                > 0
        );
        let state_4 = remove_irrelevant_txs_from_submission_material_in_state(state_3).unwrap();
        let result = state_4.get_algo_submission_material().unwrap().block.transactions;
        assert!(result.is_none());
    }

    #[test]
    fn should_remove_irrelevant_txs_from_block_in_state() {
        let dict = EvmAlgoTokenDictionary::from_str(
            &json!([{
                "evm_decimals": 18,
                "algo_decimals": 18,
                "algo_asset_id": 27165954,
                "evm_address": "0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8",
                "evm_symbol": "EVM",
                "algo_symbol": "ALGO",
            }])
            .to_string(),
        )
        .unwrap();
        let db = get_test_database();
        let submission_material = get_sample_submission_material_n(0);
        let num_txs_before = submission_material.block.transactions.clone().unwrap().len();
        let state_1 = AlgoState::init(&db);
        let state_2 = state_1.add_evm_algo_dictionary(dict).unwrap();
        let state_3 = state_2.add_algo_submission_material(&submission_material).unwrap();
        assert!(
            state_3
                .get_algo_submission_material()
                .unwrap()
                .block
                .transactions
                .unwrap()
                .len()
                > 0
        );
        let state_4 = remove_irrelevant_txs_from_submission_material_in_state(state_3).unwrap();
        let num_txs_after = state_4
            .get_algo_submission_material()
            .unwrap()
            .block
            .transactions
            .unwrap()
            .len();
        let expected_num_txs = 25;
        assert_ne!(num_txs_before, num_txs_after);
        assert_eq!(num_txs_after, expected_num_txs);
    }
}
