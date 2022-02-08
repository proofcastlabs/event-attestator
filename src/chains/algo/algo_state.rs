use rust_algorand::AlgorandBlock;

use crate::{
    chains::{algo::algo_database_utils::AlgoDbUtils, eth::eth_database_utils::EthDbUtils},
    int_on_algo::algo::int_tx_info::IntOnAlgoIntTxInfos,
    dictionaries::evm_algo::EvmAlgoTokenDictionary,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Clone, PartialEq, Eq)]
pub struct AlgoState<'a, D: DatabaseInterface> {
    db: &'a D,
    algo_block: Option<AlgorandBlock>,
    pub eth_db_utils: EthDbUtils<'a, D>,
    pub algo_db_utils: AlgoDbUtils<'a, D>,
    int_on_algo_int_tx_infos: IntOnAlgoIntTxInfos,
    pub evm_algo_token_dictionary: Option<EvmAlgoTokenDictionary>,
}

impl<'a, D: DatabaseInterface> AlgoState<'a, D> {
    pub fn init(db: &'a D) -> Self {
        Self {
            db,
            algo_block: None,
            evm_algo_token_dictionary: None,
            eth_db_utils: EthDbUtils::new(db),
            algo_db_utils: AlgoDbUtils::new(db),
            int_on_algo_int_tx_infos: IntOnAlgoIntTxInfos::default(),
        }
    }

    pub fn init_with_empty_dictionary(db: &'a D) -> Self {
        Self {
            db,
            algo_block: None,
            eth_db_utils: EthDbUtils::new(db),
            algo_db_utils: AlgoDbUtils::new(db),
            int_on_algo_int_tx_infos: IntOnAlgoIntTxInfos::default(),
            evm_algo_token_dictionary: Some(EvmAlgoTokenDictionary::default()),
        }
    }

    fn get_no_overwrite_err(item: &str) -> String {
        format!("Cannot add {} to `AlgoState` - one already exists!", item)
    }

    fn get_not_in_state_err(item: &str) -> String {
        format!("Cannot get {} from `AlgoState` - none exists!", item)
    }

    fn add_int_on_algo_int_tx_infos(mut self, infos: IntOnAlgoIntTxInfos) -> Result<Self> {
        self.int_on_algo_int_tx_infos = infos;
        Ok(self)
    }

    fn get_int_on_algo_int_tx_infos(&self) -> IntOnAlgoIntTxInfos {
        self.int_on_algo_int_tx_infos.clone()
    }

    pub fn update_submitted_block(mut self, block: &AlgorandBlock) -> Result<Self> {
        self.algo_block = Some(block.clone());
        Ok(self)
    }

    pub fn add_submitted_algo_block(self, block: &AlgorandBlock) -> Result<Self> {
        if self.get_submitted_algo_block().is_ok() {
            Err(Self::get_no_overwrite_err("algo block").into())
        } else {
            self.update_submitted_block(block)
        }
    }

    pub fn get_submitted_algo_block(&self) -> Result<AlgorandBlock> {
        match self.algo_block {
            Some(ref block) => Ok(block.clone()),
            None => Err(Self::get_not_in_state_err("algo block").into()),
        }
    }

    pub fn add_evm_algo_dictionary(mut self, dictionary: EvmAlgoTokenDictionary) -> Result<Self> {
        if self.get_evm_algo_token_dictionary().is_ok() {
            Err(Self::get_no_overwrite_err("evm_algo_token_dictionary").into())
        } else {
            self.evm_algo_token_dictionary = Some(dictionary);
            Ok(self)
        }
    }

    pub fn get_evm_algo_token_dictionary(&self) -> Result<EvmAlgoTokenDictionary> {
        match &self.evm_algo_token_dictionary {
            Some(dict) => Ok(dict.clone()),
            None => Err("No `EvmAlgoTokenDictionary` in state!".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chains::algo::test_utils::get_sample_block_n, errors::AppError, test_utils::get_test_database};

    #[test]
    fn should_put_algo_block_in_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let block = AlgorandBlock::default();
        let result = state.add_submitted_algo_block(&block);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_algo_block_from_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let block = AlgorandBlock::default();
        let updated_state = state.add_submitted_algo_block(&block.clone()).unwrap();
        let result = updated_state.get_submitted_algo_block().unwrap();
        assert_eq!(result, block);
    }

    #[test]
    fn should_not_overwrite_algo_block_in_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let block = AlgorandBlock::default();
        let updated_state = state.add_submitted_algo_block(&block).unwrap();
        let expected_error = "Cannot add algo block to `AlgoState` - one already exists!";
        match updated_state.add_submitted_algo_block(&block) {
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
        match state.get_submitted_algo_block() {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn update_submitted_block_should_allow_block_to_be_overwritten() {
        let db = get_test_database();
        let block_1 = get_sample_block_n(0);
        let block_2 = get_sample_block_n(1);
        let state_1 = AlgoState::init(&db);
        let state_2 = state_1.add_submitted_algo_block(&block_1).unwrap();
        assert_eq!(state_2.get_submitted_algo_block().unwrap(), block_1);
        let state_3 = state_2.update_submitted_block(&block_2).unwrap();
        let result = state_3.get_submitted_algo_block().unwrap();
        assert_eq!(result, block_2);
    }
}
