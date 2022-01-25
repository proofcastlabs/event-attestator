use rust_algorand::AlgorandBlock;

use crate::{chains::algo::algo_state::AlgoState, traits::DatabaseInterface, types::Result};

const NO_PARENT_ERROR: &str = "ALGO block rejected - no parent exists in database!";

fn check_submitted_block_round_is_subsequent(
    submitted_block: &AlgorandBlock,
    latest_block_from_db: &AlgorandBlock,
) -> Result<()> {
    info!("✔ Checking if submitted ALGO block round is subsequent...");
    if submitted_block.round() == latest_block_from_db.round() + 1 {
        info!("✔ Submitted ALGO block IS subsequent to latest ALGO block in db!");
        Ok(())
    } else {
        Err(NO_PARENT_ERROR.into())
    }
}

fn check_submitted_block_hash_is_subsequent(
    submitted_block: &AlgorandBlock,
    latest_block_from_db: &AlgorandBlock,
) -> Result<()> {
    info!("✔ Checking if submitted ALGO block hash is subsequent...");
    if submitted_block.get_previous_block_hash()? == latest_block_from_db.hash()? {
        info!("✔ Submitted ALGO block hash IS subsequent to latest ALGO block in db!");
        Ok(())
    } else {
        Err(NO_PARENT_ERROR.into())
    }
}

pub fn check_submitted_block_is_subsequent_and_return_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    let submitted_block = state.get_submitted_algo_block()?;
    let latest_block_from_db = state.algo_db_utils.get_latest_block()?;
    check_submitted_block_round_is_subsequent(&submitted_block, &latest_block_from_db)
        .and_then(|_| check_submitted_block_hash_is_subsequent(&submitted_block, &latest_block_from_db))
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::algo::{algo_database_utils::AlgoDbUtils, test_utils::get_sample_block_n},
        errors::AppError,
        test_utils::get_test_database,
    };

    #[test]
    fn should_check_submitted_algo_block_is_subsequent() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let state_0 = AlgoState::init(&db);
        let block_1 = get_sample_block_n(0);
        let block_2 = get_sample_block_n(1);
        let state_1 = state_0.add_submitted_algo_block(&block_2).unwrap();
        db_utils.put_latest_block_in_db(&block_1).unwrap();
        let result = check_submitted_block_is_subsequent_and_return_state(state_1);
        assert!(result.is_ok());
    }

    #[test]
    fn non_subsequent_block_should_fail_subsequency_check() {
        let db = get_test_database();
        let db_utils = AlgoDbUtils::new(&db);
        let state_0 = AlgoState::init(&db);
        let block_1 = get_sample_block_n(0);
        let block_2 = get_sample_block_n(1);
        let state_1 = state_0.add_submitted_algo_block(&block_1).unwrap();
        db_utils.put_latest_block_in_db(&block_2).unwrap();
        match check_submitted_block_is_subsequent_and_return_state(state_1) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, NO_PARENT_ERROR),
            Err(_) => panic!("Wrong error recevied!"),
        };
    }
}
