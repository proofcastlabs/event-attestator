use ethereum_types::H256 as EthHash;
use crate::{
    types::Result,
    traits::DatabaseInterface,
    chains::{
        btc::utxo_manager::utxo_types::BtcUtxosAndValues,
        eth::{
            eth_redeem_info::RedeemInfos,
            eth_submission_material::EthSubmissionMaterial,
        },
    },
    btc_on_eth::{
        btc::btc_types::BtcTransactions,
        utils::{
            get_not_in_state_err,
            get_no_overwrite_state_err,
        },
    },
};

#[derive(Clone, PartialEq, Eq)]
pub struct EthState<D: DatabaseInterface> {
    pub db: D,
    pub misc: Option<String>,
    pub btc_on_eth_redeem_infos: RedeemInfos,
    pub btc_transactions: Option<BtcTransactions>,
    pub btc_utxos_and_values: Option<BtcUtxosAndValues>,
    pub eth_submission_material: Option<EthSubmissionMaterial>,
}

impl<D> EthState<D> where D: DatabaseInterface {
    pub fn init(db: D) -> EthState<D> {
        EthState {
            db,
            misc: None,
            btc_transactions: None,
            btc_utxos_and_values: None,
            eth_submission_material: None,
            btc_on_eth_redeem_infos: RedeemInfos::new(&vec![]),
        }
    }

    pub fn add_eth_submission_material(mut self, eth_submission_material: EthSubmissionMaterial) -> Result<EthState<D>> {
        match self.eth_submission_material {
            Some(_) => Err(get_no_overwrite_state_err("eth_submission_material").into()),
            None => {
                self.eth_submission_material = Some(eth_submission_material);
                Ok(self)
            }
        }
    }

    pub fn add_btc_on_eth_redeem_infos(self, mut infos: RedeemInfos) -> Result<EthState<D>> {
        let mut new_infos = self.btc_on_eth_redeem_infos.clone().0;
        new_infos.append(&mut infos.0);
        self.replace_btc_on_eth_redeem_infos(RedeemInfos::new(&new_infos))
    }

    pub fn replace_btc_on_eth_redeem_infos(mut self, replacements: RedeemInfos) -> Result<EthState<D>> {
        self.btc_on_eth_redeem_infos = replacements;
        Ok(self)
    }

    pub fn add_misc_string_to_state(mut self, misc_string: String) -> Result<EthState<D>> {
        match self.misc {
            Some(_) => Err(get_no_overwrite_state_err("misc_string").into()),
            None => {
                self.misc = Some(misc_string);
                Ok(self)
            }
        }
    }

    pub fn add_btc_transactions(mut self, btc_transactions: BtcTransactions) -> Result<EthState<D>> {
        match self.btc_transactions {
            Some(_) => Err(get_no_overwrite_state_err("btc_transaction").into()),
            None => {
                self.btc_transactions = Some(btc_transactions);
                Ok(self)
            }
        }
    }

    pub fn add_btc_utxos_and_values(mut self, btc_utxos_and_values: BtcUtxosAndValues) -> Result<EthState<D>> {
        match self.btc_utxos_and_values {
            Some(_) => Err(get_no_overwrite_state_err("btc_utxos_and_values").into()),
            None => {
                self.btc_utxos_and_values = Some(btc_utxos_and_values);
                Ok(self)
            }
        }
    }

    pub fn update_eth_submission_material(
        mut self,
        new_eth_submission_material: EthSubmissionMaterial
    ) -> Result<EthState<D>> {
        self.eth_submission_material = Some(new_eth_submission_material);
        Ok(self)
    }

    pub fn get_eth_submission_material(&self) -> Result<&EthSubmissionMaterial> {
        match &self.eth_submission_material {
            Some(eth_submission_material) => Ok(&eth_submission_material),
            None => Err(get_not_in_state_err("eth_submission_material").into())
        }
    }

    pub fn get_misc_string(&self) -> Result<String> {
        match &self.misc {
            None => Ok("".to_string()),
            Some(misc) => Ok(misc.to_string()),
        }
    }

    pub fn get_parent_hash(&self) -> Result<EthHash> {
        Ok(self.get_eth_submission_material()?.block.parent_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        errors::AppError,
        test_utils::get_test_database,
        btc_on_eth::eth::eth_test_utils::{
            get_expected_block,
            get_expected_receipt,
            SAMPLE_RECEIPT_INDEX,
            get_sample_eth_submission_material,
            get_sample_eth_submission_material_n,
            get_valid_state_with_block_and_receipts,
        },
    };

    #[test]
    fn should_fail_to_get_eth_submission_material_in_state() {
        let expected_error = get_not_in_state_err("eth_submission_material");
        let initial_state = EthState::init(get_test_database());
        match initial_state.get_eth_submission_material() {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Ok(_) => panic!("Eth block should not be in state yet!"),
            _ => panic!("Wrong error received!")
        };
    }

    #[test]
    fn should_add_eth_submission_material_state() {
        let expected_error = get_not_in_state_err("eth_submission_material");
        let eth_submission_material = get_sample_eth_submission_material();
        let initial_state = EthState::init(get_test_database());
        match initial_state.get_eth_submission_material() {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Ok(_) => panic!("Eth block should not be in state yet!"),
            _ => panic!("Wrong error received!")
        };
        let updated_state = initial_state.add_eth_submission_material(eth_submission_material).unwrap();
        match updated_state.get_eth_submission_material() {
            Ok(block_and_receipt) => {
                let block = block_and_receipt.block.clone();
                let receipt = block_and_receipt.receipts.0[SAMPLE_RECEIPT_INDEX].clone();
                let expected_block = get_expected_block();
                let expected_receipt = get_expected_receipt();
                assert_eq!(block, expected_block);
                assert_eq!(receipt, expected_receipt);
            }
            _ => panic!("Eth block & receipts should be in state!"),
        }
    }

    #[test]
    fn should_err_when_overwriting_eth_submission_material_in_state() {
        let expected_error = get_no_overwrite_state_err("eth_submission_material");
        let eth_submission_material = get_sample_eth_submission_material();
        let initial_state = EthState::init(get_test_database());
        let updated_state = initial_state.add_eth_submission_material(eth_submission_material.clone()).unwrap();
        match updated_state.add_eth_submission_material(eth_submission_material) {
            Ok(_) => panic!("Overwriting state should not have succeeded!"),
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            _ => panic!("Wrong error recieved!")
        }
    }

    #[test]
    fn should_update_eth_submission_material() {
        let eth_submission_material_1 = get_sample_eth_submission_material_n(0).unwrap();
        let eth_submission_material_2 = get_sample_eth_submission_material_n(1).unwrap();
        let initial_state = EthState::init(get_test_database());
        let updated_state = initial_state.add_eth_submission_material(eth_submission_material_1).unwrap();
        let initial_state_block_num = updated_state.get_eth_submission_material().unwrap().block.number;
        let final_state = updated_state.update_eth_submission_material(eth_submission_material_2).unwrap();
        let final_state_block_number = final_state.get_eth_submission_material().unwrap().block.number;
        assert_ne!(final_state_block_number, initial_state_block_num);
    }

    #[test]
    fn should_get_eth_parent_hash() {
        let expected_result = get_sample_eth_submission_material().block.parent_hash;
        let state = get_valid_state_with_block_and_receipts().unwrap();
        let result = state.get_parent_hash().unwrap();
        assert_eq!(result, expected_result);
    }
}
