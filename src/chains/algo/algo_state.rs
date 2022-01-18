use rust_algorand::AlgorandBlock;

use crate::{
    chains::{algo::algo_database_utils::AlgoDbUtils, eth::eth_database_utils::EthDbUtils},
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Clone, PartialEq, Eq)]
pub struct AlgoState<'a, D: DatabaseInterface> {
    db: &'a D,
    algo_block: Option<AlgorandBlock>,
    pub eth_db_utils: EthDbUtils<'a, D>,
    pub algo_db_utils: AlgoDbUtils<'a, D>,
}

impl<'a, D: DatabaseInterface> AlgoState<'a, D> {
    pub fn init(db: &'a D) -> Self {
        Self {
            db,
            algo_block: None,
            eth_db_utils: EthDbUtils::new(db),
            algo_db_utils: AlgoDbUtils::new(db),
        }
    }

    fn get_no_overwrite_err(item: &str) -> String {
        format!("Cannot add {} to `AlgoState` - one already exists!", item)
    }

    fn get_not_in_state_err(item: &str) -> String {
        format!("Cannot get {} from `AlgoState` - none exists!", item)
    }

    pub fn add_algo_block(mut self, block: &AlgorandBlock) -> Result<Self> {
        if self.get_algo_block().is_ok() {
            Err(Self::get_no_overwrite_err("algo block").into())
        } else {
            self.algo_block = Some(block.clone());
            Ok(self)
        }
    }

    pub fn get_algo_block(&self) -> Result<AlgorandBlock> {
        match self.algo_block {
            Some(ref block) => Ok(block.clone()),
            None => Err(Self::get_not_in_state_err("algo block").into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{errors::AppError, test_utils::get_test_database};

    #[test]
    fn should_put_algo_block_in_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let block = AlgorandBlock::default();
        let result = state.add_algo_block(&block);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_algo_block_from_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let block = AlgorandBlock::default();
        let updated_state = state.add_algo_block(&block.clone()).unwrap();
        let result = updated_state.get_algo_block().unwrap();
        assert_eq!(result, block);
    }

    #[test]
    fn should_not_overwrite_algo_block_in_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let block = AlgorandBlock::default();
        let updated_state = state.add_algo_block(&block).unwrap();
        let expected_error = "Cannot add algo block to `AlgoState` - one already exists!";
        match updated_state.add_algo_block(&block) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_fail_to_get_block_if_not_in_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let expected_error = "Cannot get algo block from `AlgoState` - none exists!";
        match state.get_algo_block() {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
