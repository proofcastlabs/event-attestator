use ethereum_types::H256 as EthHash;

use crate::{
    btc_on_eth::BtcOnEthBtcTxInfos,
    btc_on_int::BtcOnIntBtcTxInfos,
    chains::{
        algo::{algo_database_utils::AlgoDbUtils, algo_signed_group_txs::AlgoSignedGroupTxs},
        btc::{
            btc_database_utils::BtcDbUtils,
            btc_types::BtcTransactions,
            utxo_manager::utxo_types::BtcUtxosAndValues,
        },
        eos::{eos_crypto::eos_transaction::EosSignedTransactions, eos_database_utils::EosDbUtils},
        eth::{
            eth_crypto::eth_transaction::EthTransactions,
            eth_database_utils::{EthDbUtils, EvmDbUtils},
            eth_submission_material::EthSubmissionMaterial,
        },
    },
    dictionaries::{eos_eth::EosEthTokenDictionary, eth_evm::EthEvmTokenDictionary, evm_algo::EvmAlgoTokenDictionary},
    eos_on_eth::EosOnEthEthTxInfos,
    eos_on_int::EosOnIntEosTxInfos,
    erc20_on_eos::Erc20OnEosPegInInfos,
    erc20_on_evm::{eth::evm_tx_info::Erc20OnEvmEvmTxInfos, evm::eth_tx_info::Erc20OnEvmEthTxInfos},
    erc20_on_int::{eth::int_tx_info::Erc20OnIntIntTxInfos, int::eth_tx_info::Erc20OnIntEthTxInfos},
    int_on_algo::int::algo_tx_info::IntOnAlgoAlgoTxInfos,
    int_on_eos::int::eos_tx_info::IntOnEosEosTxInfos,
    int_on_evm::{evm::int_tx_info::IntOnEvmIntTxInfos, int::evm_tx_info::IntOnEvmEvmTxInfos},
    traits::DatabaseInterface,
    types::Result,
    utils::{get_no_overwrite_state_err, get_not_in_state_err},
};

// FIXME We can move the core specific setters & getters of this into their own mods!
// FIXME Use a macro to generate a lot of this!

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EthState<'a, D: DatabaseInterface> {
    pub db: &'a D,
    pub misc: Option<String>,
    pub eth_db_utils: EthDbUtils<'a, D>,
    pub evm_db_utils: EvmDbUtils<'a, D>,
    pub btc_db_utils: BtcDbUtils<'a, D>,
    pub eos_db_utils: EosDbUtils<'a, D>,
    pub algo_db_utils: AlgoDbUtils<'a, D>,
    pub btc_transactions: Option<BtcTransactions>,
    pub algo_signed_group_txs: AlgoSignedGroupTxs,
    pub int_on_evm_int_signed_txs: EthTransactions,
    pub int_on_evm_evm_signed_txs: EthTransactions,
    pub eos_on_int_eos_tx_infos: EosOnIntEosTxInfos,
    pub int_on_evm_evm_tx_infos: IntOnEvmEvmTxInfos,
    pub int_on_evm_int_tx_infos: IntOnEvmIntTxInfos,
    pub eos_on_eth_eth_tx_infos: EosOnEthEthTxInfos,
    pub int_on_eos_eos_tx_infos: IntOnEosEosTxInfos,
    pub btc_on_int_btc_tx_infos: BtcOnIntBtcTxInfos,
    pub btc_on_eth_btc_tx_infos: BtcOnEthBtcTxInfos,
    pub erc20_on_evm_evm_signed_txs: EthTransactions,
    pub erc20_on_evm_eth_signed_txs: EthTransactions,
    pub erc20_on_int_int_signed_txs: EthTransactions,
    pub erc20_on_int_eth_signed_txs: EthTransactions,
    pub int_on_algo_algo_tx_infos: IntOnAlgoAlgoTxInfos,
    pub erc20_on_evm_eth_tx_infos: Erc20OnEvmEthTxInfos,
    pub erc20_on_evm_evm_tx_infos: Erc20OnEvmEvmTxInfos,
    pub erc20_on_int_eth_tx_infos: Erc20OnIntEthTxInfos,
    pub erc20_on_int_int_tx_infos: Erc20OnIntIntTxInfos,
    pub erc20_on_eos_peg_in_infos: Erc20OnEosPegInInfos,
    pub eos_transactions: Option<EosSignedTransactions>,
    pub btc_utxos_and_values: Option<BtcUtxosAndValues>,
    pub eth_submission_material: Option<EthSubmissionMaterial>,
    pub eos_eth_token_dictionary: Option<EosEthTokenDictionary>,
    pub eth_evm_token_dictionary: Option<EthEvmTokenDictionary>,
    pub evm_algo_token_dictionary: Option<EvmAlgoTokenDictionary>,
}

impl<'a, D: DatabaseInterface> EthState<'a, D> {
    pub fn init(db: &'a D) -> EthState<'a, D> {
        EthState {
            db,
            misc: None,
            btc_transactions: None,
            eos_transactions: None,
            btc_utxos_and_values: None,
            eth_submission_material: None,
            eth_evm_token_dictionary: None,
            eos_eth_token_dictionary: None,
            evm_algo_token_dictionary: None,
            eth_db_utils: EthDbUtils::new(db),
            evm_db_utils: EvmDbUtils::new(db),
            eos_db_utils: EosDbUtils::new(db),
            btc_db_utils: BtcDbUtils::new(db),
            algo_db_utils: AlgoDbUtils::new(db),
            algo_signed_group_txs: AlgoSignedGroupTxs::new(vec![]),
            int_on_evm_int_signed_txs: EthTransactions::new(vec![]),
            int_on_evm_evm_signed_txs: EthTransactions::new(vec![]),
            btc_on_int_btc_tx_infos: BtcOnIntBtcTxInfos::new(vec![]),
            int_on_evm_evm_tx_infos: IntOnEvmEvmTxInfos::new(vec![]),
            eos_on_int_eos_tx_infos: EosOnIntEosTxInfos::new(vec![]),
            int_on_eos_eos_tx_infos: IntOnEosEosTxInfos::new(vec![]),
            int_on_evm_int_tx_infos: IntOnEvmIntTxInfos::new(vec![]),
            btc_on_eth_btc_tx_infos: BtcOnEthBtcTxInfos::new(vec![]),
            eos_on_eth_eth_tx_infos: EosOnEthEthTxInfos::new(vec![]),
            erc20_on_evm_evm_signed_txs: EthTransactions::new(vec![]),
            erc20_on_evm_eth_signed_txs: EthTransactions::new(vec![]),
            erc20_on_int_int_signed_txs: EthTransactions::new(vec![]),
            erc20_on_int_eth_signed_txs: EthTransactions::new(vec![]),
            int_on_algo_algo_tx_infos: IntOnAlgoAlgoTxInfos::default(),
            erc20_on_evm_evm_tx_infos: Erc20OnEvmEvmTxInfos::new(vec![]),
            erc20_on_evm_eth_tx_infos: Erc20OnEvmEthTxInfos::new(vec![]),
            erc20_on_int_eth_tx_infos: Erc20OnIntEthTxInfos::new(vec![]),
            erc20_on_int_int_tx_infos: Erc20OnIntIntTxInfos::new(vec![]),
            erc20_on_eos_peg_in_infos: Erc20OnEosPegInInfos::new(vec![]),
        }
    }

    pub fn get_eth_evm_token_dictionary(&self) -> Result<&EthEvmTokenDictionary> {
        match self.eth_evm_token_dictionary {
            Some(ref dictionary) => Ok(dictionary),
            None => Err(get_not_in_state_err("eth_evm_token_dictionary").into()),
        }
    }

    pub fn get_evm_algo_token_dictionary(&self) -> Result<&EvmAlgoTokenDictionary> {
        match self.evm_algo_token_dictionary {
            Some(ref dictionary) => Ok(dictionary),
            None => Err(get_not_in_state_err("evm_algo_token_dictionary").into()),
        }
    }

    pub fn add_erc20_on_evm_eth_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.erc20_on_evm_eth_signed_txs.is_empty() {
            self.erc20_on_evm_eth_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("erc20_on_evm_eth_signed_txs").into())
        }
    }

    pub fn add_erc20_on_int_eth_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.erc20_on_int_eth_signed_txs.is_empty() {
            self.erc20_on_int_eth_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("erc20_on_int_eth_signed_txs").into())
        }
    }

    pub fn add_erc20_on_int_int_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.erc20_on_int_int_signed_txs.is_empty() {
            self.erc20_on_int_int_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("erc20_on_int_int_signed_txs").into())
        }
    }

    pub fn add_int_on_evm_int_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.int_on_evm_int_signed_txs.is_empty() {
            self.int_on_evm_int_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("int_on_evm_int_signed_txs").into())
        }
    }

    pub fn add_int_on_evm_evm_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.int_on_evm_evm_signed_txs.is_empty() {
            self.int_on_evm_evm_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("int_on_evm_evm_signed_txs").into())
        }
    }

    pub fn add_int_on_evm_evm_tx_infos(self, mut infos: IntOnEvmEvmTxInfos) -> Result<Self> {
        let mut new_infos = self.int_on_evm_evm_tx_infos.0.clone();
        new_infos.append(&mut infos.0);
        self.replace_int_on_evm_evm_tx_infos(IntOnEvmEvmTxInfos::new(new_infos))
    }

    pub fn add_int_on_evm_int_tx_infos(self, mut infos: IntOnEvmIntTxInfos) -> Result<Self> {
        let mut new_infos = self.int_on_evm_int_tx_infos.0.clone();
        new_infos.append(&mut infos.0);
        self.replace_int_on_evm_int_tx_infos(IntOnEvmIntTxInfos::new(new_infos))
    }

    pub fn add_int_on_algo_algo_tx_infos(self, mut infos: IntOnAlgoAlgoTxInfos) -> Result<Self> {
        let mut new_infos = self.int_on_algo_algo_tx_infos.0.clone();
        new_infos.append(&mut infos.0);
        self.replace_int_on_algo_algo_tx_infos(IntOnAlgoAlgoTxInfos::new(new_infos))
    }

    pub fn add_erc20_on_evm_eth_tx_infos(self, mut infos: Erc20OnEvmEthTxInfos) -> Result<Self> {
        let mut new_infos = self.erc20_on_evm_eth_tx_infos.0.clone();
        new_infos.append(&mut infos.0);
        self.replace_erc20_on_evm_eth_tx_infos(Erc20OnEvmEthTxInfos::new(new_infos))
    }

    pub fn add_eth_submission_material(mut self, eth_submission_material: EthSubmissionMaterial) -> Result<Self> {
        match self.eth_submission_material {
            Some(_) => Err(get_no_overwrite_state_err("eth_submission_material").into()),
            None => {
                self.eth_submission_material = Some(eth_submission_material);
                Ok(self)
            },
        }
    }

    pub fn add_btc_on_eth_btc_tx_infos(self, mut infos: BtcOnEthBtcTxInfos) -> Result<Self> {
        let mut new_infos = self.btc_on_eth_btc_tx_infos.clone().0;
        new_infos.append(&mut infos.0);
        self.replace_btc_on_eth_btc_tx_infos(BtcOnEthBtcTxInfos::new(new_infos))
    }

    pub fn add_btc_on_int_btc_tx_infos(self, mut infos: BtcOnIntBtcTxInfos) -> Result<Self> {
        let mut new_infos = self.btc_on_int_btc_tx_infos.clone().0;
        new_infos.append(&mut infos.0);
        self.replace_btc_on_int_btc_tx_infos(BtcOnIntBtcTxInfos::new(new_infos))
    }

    pub fn add_erc20_on_eos_peg_in_infos(self, mut infos: Erc20OnEosPegInInfos) -> Result<Self> {
        let mut new_infos = self.erc20_on_eos_peg_in_infos.clone().0;
        new_infos.append(&mut infos.0);
        self.replace_erc20_on_eos_peg_in_infos(Erc20OnEosPegInInfos::new(new_infos))
    }

    pub fn add_eos_on_eth_eth_tx_infos(self, mut infos: EosOnEthEthTxInfos) -> Result<Self> {
        let mut new_infos = self.eos_on_eth_eth_tx_infos.clone().0;
        new_infos.append(&mut infos.0);
        self.replace_eos_on_eth_eth_tx_infos(EosOnEthEthTxInfos::new(new_infos))
    }

    pub fn add_eos_on_int_eos_tx_infos(self, mut infos: EosOnIntEosTxInfos) -> Result<Self> {
        let mut new_infos = self.eos_on_int_eos_tx_infos.clone().0;
        new_infos.append(&mut infos.0);
        self.replace_eos_on_int_eos_tx_infos(EosOnIntEosTxInfos::new(new_infos))
    }

    pub fn add_erc20_on_evm_evm_tx_infos(self, mut infos: Erc20OnEvmEvmTxInfos) -> Result<Self> {
        let mut new_infos = self.erc20_on_evm_evm_tx_infos.0.clone();
        new_infos.append(&mut infos.0);
        self.replace_erc20_on_evm_evm_tx_infos(Erc20OnEvmEvmTxInfos::new(new_infos))
    }

    pub fn add_int_on_eos_eos_tx_infos(self, mut infos: IntOnEosEosTxInfos) -> Result<Self> {
        let mut new_infos = self.int_on_eos_eos_tx_infos.0.clone();
        new_infos.append(&mut infos.0);
        self.replace_int_on_eos_eos_tx_infos(IntOnEosEosTxInfos::new(new_infos))
    }

    pub fn add_erc20_on_int_int_tx_infos(self, mut infos: Erc20OnIntIntTxInfos) -> Result<Self> {
        let mut new_infos = self.erc20_on_int_int_tx_infos.0.clone();
        new_infos.append(&mut infos.0);
        self.replace_erc20_on_int_int_tx_infos(Erc20OnIntIntTxInfos::new(new_infos))
    }

    pub fn add_erc20_on_int_eth_tx_infos(self, mut infos: Erc20OnIntEthTxInfos) -> Result<Self> {
        let mut new_infos = self.erc20_on_int_eth_tx_infos.0.clone();
        new_infos.append(&mut infos.0);
        self.replace_erc20_on_int_eth_tx_infos(Erc20OnIntEthTxInfos::new(new_infos))
    }

    pub fn replace_erc20_on_int_int_tx_infos(mut self, replacements: Erc20OnIntIntTxInfos) -> Result<Self> {
        self.erc20_on_int_int_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_int_on_eos_eos_tx_infos(mut self, replacements: IntOnEosEosTxInfos) -> Result<Self> {
        self.int_on_eos_eos_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_erc20_on_int_eth_tx_infos(mut self, replacements: Erc20OnIntEthTxInfos) -> Result<Self> {
        self.erc20_on_int_eth_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_btc_on_int_btc_tx_infos(mut self, replacements: BtcOnIntBtcTxInfos) -> Result<Self> {
        self.btc_on_int_btc_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_btc_on_eth_btc_tx_infos(mut self, replacements: BtcOnEthBtcTxInfos) -> Result<Self> {
        self.btc_on_eth_btc_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_erc20_on_eos_peg_in_infos(mut self, replacements: Erc20OnEosPegInInfos) -> Result<Self> {
        self.erc20_on_eos_peg_in_infos = replacements;
        Ok(self)
    }

    pub fn replace_eos_on_eth_eth_tx_infos(mut self, replacements: EosOnEthEthTxInfos) -> Result<Self> {
        self.eos_on_eth_eth_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_eos_on_int_eos_tx_infos(mut self, replacements: EosOnIntEosTxInfos) -> Result<Self> {
        self.eos_on_int_eos_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_erc20_on_evm_eth_tx_infos(mut self, replacements: Erc20OnEvmEthTxInfos) -> Result<Self> {
        self.erc20_on_evm_eth_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_erc20_on_evm_evm_tx_infos(mut self, replacements: Erc20OnEvmEvmTxInfos) -> Result<Self> {
        self.erc20_on_evm_evm_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_int_on_evm_evm_tx_infos(mut self, replacements: IntOnEvmEvmTxInfos) -> Result<Self> {
        self.int_on_evm_evm_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_int_on_evm_int_tx_infos(mut self, replacements: IntOnEvmIntTxInfos) -> Result<Self> {
        self.int_on_evm_int_tx_infos = replacements;
        Ok(self)
    }

    pub fn replace_int_on_algo_algo_tx_infos(mut self, replacements: IntOnAlgoAlgoTxInfos) -> Result<Self> {
        self.int_on_algo_algo_tx_infos = replacements;
        Ok(self)
    }

    pub fn add_btc_transactions(mut self, btc_transactions: BtcTransactions) -> Result<Self> {
        match self.btc_transactions {
            Some(_) => Err(get_no_overwrite_state_err("btc_transaction").into()),
            None => {
                self.btc_transactions = Some(btc_transactions);
                Ok(self)
            },
        }
    }

    pub fn add_eos_transactions(mut self, eos_transactions: EosSignedTransactions) -> Result<Self> {
        match self.eos_transactions {
            Some(_) => Err(get_no_overwrite_state_err("eos_transaction").into()),
            None => {
                self.eos_transactions = Some(eos_transactions);
                Ok(self)
            },
        }
    }

    pub fn add_erc20_on_evm_evm_signed_txs(mut self, txs: EthTransactions) -> Result<Self> {
        if self.erc20_on_evm_evm_signed_txs.is_empty() {
            self.erc20_on_evm_evm_signed_txs = txs;
            Ok(self)
        } else {
            Err(get_no_overwrite_state_err("evm_transaction").into())
        }
    }

    pub fn update_eth_submission_material(
        mut self,
        new_eth_submission_material: EthSubmissionMaterial,
    ) -> Result<Self> {
        self.eth_submission_material = Some(new_eth_submission_material);
        Ok(self)
    }

    pub fn get_eth_submission_material(&self) -> Result<&EthSubmissionMaterial> {
        match self.eth_submission_material {
            Some(ref eth_submission_material) => Ok(eth_submission_material),
            None => Err(get_not_in_state_err("eth_submission_material").into()),
        }
    }

    pub fn get_parent_hash(&self) -> Result<EthHash> {
        self.get_eth_submission_material()?.get_parent_hash()
    }

    pub fn get_num_eos_txs(&self) -> usize {
        match self.eos_transactions {
            None => 0,
            Some(ref txs) => txs.len(),
        }
    }

    pub fn add_eos_eth_token_dictionary(mut self, dictionary: EosEthTokenDictionary) -> Result<Self> {
        match self.eos_eth_token_dictionary {
            Some(_) => Err(get_no_overwrite_state_err("eos_eth_token_dictionary").into()),
            None => {
                self.eos_eth_token_dictionary = Some(dictionary);
                Ok(self)
            },
        }
    }

    pub fn get_eos_eth_token_dictionary(&self) -> Result<&EosEthTokenDictionary> {
        match self.eos_eth_token_dictionary {
            Some(ref dictionary) => Ok(dictionary),
            None => Err(get_not_in_state_err("eos_eth_token_dictionary").into()),
        }
    }

    pub fn add_eth_evm_token_dictionary(mut self, dictionary: EthEvmTokenDictionary) -> Result<Self> {
        match self.eth_evm_token_dictionary {
            Some(_) => Err(get_no_overwrite_state_err("eth_evm_token_dictionary").into()),
            None => {
                self.eth_evm_token_dictionary = Some(dictionary);
                Ok(self)
            },
        }
    }

    pub fn add_evm_algo_dictionary(mut self, dictionary: EvmAlgoTokenDictionary) -> Result<Self> {
        match self.evm_algo_token_dictionary {
            Some(_) => Err(get_no_overwrite_state_err("evm_algo_token_dictionary").into()),
            None => {
                self.evm_algo_token_dictionary = Some(dictionary);
                Ok(self)
            },
        }
    }

    pub fn add_algo_signed_group_txs(mut self, txs: &AlgoSignedGroupTxs) -> Result<Self> {
        if !self.algo_signed_group_txs.is_empty() {
            Err(get_no_overwrite_state_err("algo signed group txs").into())
        } else {
            self.algo_signed_group_txs = txs.clone();
            Ok(self)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::eth_test_utils::{
            get_expected_block,
            get_expected_receipt,
            get_sample_eth_submission_material,
            get_sample_eth_submission_material_n,
            SAMPLE_RECEIPT_INDEX,
        },
        errors::AppError,
        test_utils::get_test_database,
    };

    #[test]
    fn should_fail_to_get_eth_submission_material_in_state() {
        let expected_error = get_not_in_state_err("eth_submission_material");
        let db = get_test_database();
        let initial_state = EthState::init(&db);
        match initial_state.get_eth_submission_material() {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Ok(_) => panic!("Eth block should not be in state yet!"),
            _ => panic!("Wrong error received!"),
        };
    }

    #[test]
    fn should_add_eth_submission_material_state() {
        let expected_error = get_not_in_state_err("eth_submission_material");
        let eth_submission_material = get_sample_eth_submission_material();
        let db = get_test_database();
        let initial_state = EthState::init(&db);
        match initial_state.get_eth_submission_material() {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Ok(_) => panic!("Eth block should not be in state yet!"),
            _ => panic!("Wrong error received!"),
        };
        let updated_state = initial_state
            .add_eth_submission_material(eth_submission_material)
            .unwrap();
        let submission_material = updated_state.get_eth_submission_material().unwrap();
        let block = submission_material.get_block().unwrap();
        let receipt = submission_material.receipts.0[SAMPLE_RECEIPT_INDEX].clone();
        let expected_block = get_expected_block();
        let expected_receipt = get_expected_receipt();
        assert_eq!(block, expected_block);
        assert_eq!(receipt, expected_receipt);
    }

    #[test]
    fn should_err_when_overwriting_eth_submission_material_in_state() {
        let expected_error = get_no_overwrite_state_err("eth_submission_material");
        let eth_submission_material = get_sample_eth_submission_material();
        let db = get_test_database();
        let initial_state = EthState::init(&db);
        let updated_state = initial_state
            .add_eth_submission_material(eth_submission_material.clone())
            .unwrap();
        match updated_state.add_eth_submission_material(eth_submission_material) {
            Ok(_) => panic!("Overwriting state should not have succeeded!"),
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            _ => panic!("Wrong error recieved!"),
        }
    }

    #[test]
    fn should_update_eth_submission_material() {
        let eth_submission_material_1 = get_sample_eth_submission_material_n(0).unwrap();
        let eth_submission_material_2 = get_sample_eth_submission_material_n(1).unwrap();
        let db = get_test_database();
        let initial_state = EthState::init(&db);
        let updated_state = initial_state
            .add_eth_submission_material(eth_submission_material_1)
            .unwrap();
        let initial_state_block_num = updated_state
            .get_eth_submission_material()
            .unwrap()
            .get_block_number()
            .unwrap();
        let final_state = updated_state
            .update_eth_submission_material(eth_submission_material_2)
            .unwrap();
        let final_state_block_number = final_state
            .get_eth_submission_material()
            .unwrap()
            .get_block_number()
            .unwrap();
        assert_ne!(final_state_block_number, initial_state_block_num);
    }

    #[test]
    fn should_not_allow_overwrite_of_eth_evm_token_dictionary() {
        let db = get_test_database();
        let state = EthState::init(&db);
        let dictionary = EthEvmTokenDictionary::default();
        let updated_state = state.add_eth_evm_token_dictionary(dictionary.clone()).unwrap();
        let expected_error = get_no_overwrite_state_err("eth_evm_token_dictionary");
        match updated_state.add_eth_evm_token_dictionary(dictionary) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
