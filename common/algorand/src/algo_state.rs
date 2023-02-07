use common::{
    chains::eth::{eth_crypto::eth_transaction::EthTransactions, eth_database_utils::EthDbUtils},
    dictionaries::evm_algo::EvmAlgoTokenDictionary,
    traits::DatabaseInterface,
    types::{Bytes, Result},
};

use crate::{AlgoDbUtils, AlgoRelevantAssetTxs, AlgoSubmissionMaterial};

#[derive(Clone, PartialEq, Eq)]
pub struct AlgoState<'a, D: DatabaseInterface> {
    db: &'a D,
    pub tx_infos: Bytes,
    pub eth_db_utils: EthDbUtils<'a, D>,
    pub eth_signed_txs: EthTransactions,
    pub algo_db_utils: AlgoDbUtils<'a, D>,
    pub algo_relevant_asset_txs: Option<AlgoRelevantAssetTxs>,
    pub algo_submission_material: Option<AlgoSubmissionMaterial>,
    pub evm_algo_token_dictionary: Option<EvmAlgoTokenDictionary>,
}

impl<'a, D: DatabaseInterface> AlgoState<'a, D> {
    fn init_inner(db: &'a D, evm_algo_token_dictionary: Option<EvmAlgoTokenDictionary>) -> Self {
        Self {
            db,
            tx_infos: vec![],
            evm_algo_token_dictionary,
            algo_relevant_asset_txs: None,
            algo_submission_material: None,
            eth_db_utils: EthDbUtils::new(db),
            algo_db_utils: AlgoDbUtils::new(db),
            eth_signed_txs: EthTransactions::new(vec![]), // TODO impl default
        }
    }

    pub fn add_tx_infos(mut self, bytes: Bytes) -> Self {
        info!("✔ Adding tx infos to algo state!");
        self.tx_infos = bytes;
        self
    }

    pub fn init(db: &'a D) -> Self {
        Self::init_inner(db, None)
    }

    pub fn init_with_empty_dictionary(db: &'a D) -> Self {
        Self::init_inner(db, Some(EvmAlgoTokenDictionary::default()))
    }

    fn get_no_overwrite_err(item: &str) -> String {
        format!("Cannot add {} to `AlgoState` - one already exists!", item)
    }

    fn get_not_in_state_err(item: &str) -> String {
        format!("Cannot get {} from `AlgoState` - none exists!", item)
    }

    pub fn add_eth_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        self.eth_signed_txs = txs;
        Ok(self)
    }

    pub fn update_algo_submission_material(mut self, material: &AlgoSubmissionMaterial) -> Result<Self> {
        self.algo_submission_material = Some(material.clone());
        Ok(self)
    }

    pub fn add_algo_submission_material(self, material: &AlgoSubmissionMaterial) -> Result<Self> {
        if self.get_algo_submission_material().is_ok() {
            Err(Self::get_no_overwrite_err("algo submission material").into())
        } else {
            self.update_algo_submission_material(material)
        }
    }

    pub fn get_algo_submission_material(&self) -> Result<AlgoSubmissionMaterial> {
        match self.algo_submission_material {
            Some(ref material) => Ok(material.clone()),
            None => Err(Self::get_not_in_state_err("algo submission material").into()),
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

    pub fn get_num_signed_txs(&self) -> usize {
        self.eth_signed_txs.len()
    }

    pub fn get_relevant_asset_txs(&self) -> Result<AlgoRelevantAssetTxs> {
        match &self.algo_relevant_asset_txs {
            Some(txs) => Ok(txs.clone()),
            None => Err(Self::get_not_in_state_err("algo relevant asset txs").into()),
        }
    }

    pub fn update_relevant_asset_txs(mut self, txs: &AlgoRelevantAssetTxs) -> Result<Self> {
        self.algo_relevant_asset_txs = Some(txs.clone());
        Ok(self)
    }

    pub fn add_relevant_asset_txs(self, txs: &AlgoRelevantAssetTxs) -> Result<Self> {
        if self.get_relevant_asset_txs().is_ok() {
            Err(Self::get_no_overwrite_err("algo relevant asset txs").into())
        } else {
            self.update_relevant_asset_txs(txs)
        }
    }
}

#[cfg(test)]
mod tests {
    use common::{errors::AppError, test_utils::get_test_database};

    use super::*;
    use crate::test_utils::get_sample_submission_material_n;

    #[test]
    fn should_put_algo_block_in_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let block = AlgoSubmissionMaterial::default();
        let result = state.add_algo_submission_material(&block);
        assert!(result.is_ok());
    }

    #[test]
    fn should_get_algo_block_from_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let block = AlgoSubmissionMaterial::default();
        let updated_state = state.add_algo_submission_material(&block).unwrap();
        let result = updated_state.get_algo_submission_material().unwrap();
        assert_eq!(result, block);
    }

    #[test]
    fn should_not_overwrite_algo_submission_material_in_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let block = AlgoSubmissionMaterial::default();
        let updated_state = state.add_algo_submission_material(&block).unwrap();
        let expected_error = "Cannot add algo submission material to `AlgoState` - one already exists!";
        match updated_state.add_algo_submission_material(&block) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_fail_to_get_submission_material_if_not_in_state() {
        let db = get_test_database();
        let state = AlgoState::init(&db);
        let expected_error = "Cannot get algo submission material from `AlgoState` - none exists!";
        match state.get_algo_submission_material() {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn update_submission_material_should_allow_block_to_be_overwritten() {
        let db = get_test_database();
        let block_1 = get_sample_submission_material_n(0);
        let block_2 = get_sample_submission_material_n(1);
        let state_1 = AlgoState::init(&db);
        let state_2 = state_1.add_algo_submission_material(&block_1).unwrap();
        assert_eq!(state_2.get_algo_submission_material().unwrap(), block_1);
        let state_3 = state_2.update_algo_submission_material(&block_2).unwrap();
        let result = state_3.get_algo_submission_material().unwrap();
        assert_eq!(result, block_2);
    }
}
