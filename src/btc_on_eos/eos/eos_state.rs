use eos_primitives::{
    BlockHeader as EosBlockHeader,
    ProducerSchedule as EosProducerSchedule,
};
use crate::btc_on_eos::{
    types::Result,
    errors::AppError,
    traits::DatabaseInterface,
    btc::{
        btc_types::{
            BtcTransactions,
            BtcUtxosAndValues,
        },
    },
    eos::{
        eos_types::{
            ActionProofs,
            Checksum256s,
            RedeemParams,
            ProcessedTxIds,
            EosSubmissionMaterial,
        },
    },
    utils::{
        get_not_in_state_err,
        get_no_overwrite_state_err,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EosState<D: DatabaseInterface> {
    pub db: D,
    pub producer_signature: String,
    pub action_proofs: ActionProofs,
    pub signed_txs: BtcTransactions,
    pub blockroot_merkle: Checksum256s,
    pub redeem_params: Vec<RedeemParams>,
    pub processed_tx_ids: ProcessedTxIds,
    pub block_header: Option<EosBlockHeader>,
    pub active_schedule: EosProducerSchedule,
    pub btc_utxos_and_values: Option<BtcUtxosAndValues>,
}

impl<D> EosState<D> where D: DatabaseInterface {
    pub fn init(db: D) -> EosState<D> {
        EosState {
            db,
            block_header: None,
            signed_txs: vec![],
            action_proofs: vec![],
            redeem_params: vec![],
            blockroot_merkle: vec![],
            btc_utxos_and_values: None,
            producer_signature: String::new(),
            processed_tx_ids: ProcessedTxIds::init(),
            active_schedule: EosProducerSchedule::default(),
        }
    }

    pub fn add_btc_utxos_and_values(
        mut self,
        btc_utxos_and_values: BtcUtxosAndValues,
    ) -> Result<EosState<D>> {
        match self.btc_utxos_and_values {
            Some(_) => Err(AppError::Custom(
                get_no_overwrite_state_err("btc_utxos_and_values"))
            ),
            None => {
                self.btc_utxos_and_values = Some(btc_utxos_and_values);
                Ok(self)
            }
        }
    }

    pub fn add_signed_txs(
        mut self,
        signed_txs: BtcTransactions,
    ) -> Result<EosState<D>>
        where D: DatabaseInterface
    {
        self.signed_txs = signed_txs;
        Ok(self)
    }

    pub fn add_submission_material(
        mut self,
        submission_material: EosSubmissionMaterial,
    ) -> Result<EosState<D>> {
        self.action_proofs = submission_material.action_proofs;
        self.block_header = Some(submission_material.block_header);
        self.active_schedule = submission_material.active_schedule;
        self.blockroot_merkle = submission_material.blockroot_merkle;
        self.producer_signature = submission_material.producer_signature;
        Ok(self)
    }

    pub fn add_redeem_params(
        mut self,
        redeem_params: Vec<RedeemParams>,
    ) -> Result<EosState<D>> {
        self.redeem_params = redeem_params;
        Ok(self)
    }

    pub fn add_processed_tx_ids(
        mut self,
        tx_ids: ProcessedTxIds,
    ) -> Result<Self> {
        self.processed_tx_ids = tx_ids;
        Ok(self)
    }

    pub fn get_eos_block_header(
        &self
    ) -> Result<&EosBlockHeader> {
        match &self.block_header{
            Some(block_header) => Ok(&block_header),
            None => Err(AppError::Custom(
                get_not_in_state_err("block_header"))
            )
        }
    }

    pub fn replace_redeem_params(
        mut self,
        replacement_params: Vec<RedeemParams>,
    ) -> Result<EosState<D>> {
        info!("✔ Replacing redeem params in state...");
        self.redeem_params = replacement_params;
        Ok(self)
    }

    pub fn replace_action_proofs(
        mut self,
        replacements: ActionProofs,
    ) -> Result<EosState<D>> {
        info!("✔ Replacing `action_proofs` in state...");
        self.action_proofs = replacements;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    /* TODO Reinstate!
    use super::*;
    use crate::btc_on_eos::{
        eos::eos_test_utils::get_valid_initial_eos_state,
        test_utils::{
            get_sample_config,
            TEMPORARY_DATABASE_PATH,
        },
    };

    #[test]
    fn should_fail_to_get_non_existent_config_in_state() {
        let expected_error = get_not_in_state_err("config");
        let state = get_valid_initial_eos_state(true)
            .unwrap();
        match state.get_config() {
            Err(AppError::Custom(e)) => assert!(e == expected_error),
            Ok(_) => panic!("Config should not be in state"),
            Err(_) => panic!("Wrong error recieved!")
        }
    }

    #[test]
    fn should_add_config_in_state() {
        let config = get_sample_config();
        let state = get_valid_initial_eos_state(true)
            .unwrap();
        if let Ok(_) = state.get_config() {
            panic!("Config should not be in state!")
        }
        let resulting_state = state.add_config(config)
            .unwrap();
        match resulting_state.get_config() {
            Ok(config) => {
                assert!(config.DATABASE_PATH == TEMPORARY_DATABASE_PATH)
            },
            _ => panic!("Config should be in state!")
        }
    }

    #[test]
    fn should_not_be_able_to_overwrite_config_in_state() {
        let expected_error = get_no_overwrite_state_err("config");
        let config = get_sample_config();
        let state = get_valid_initial_eos_state(true)
            .unwrap();
        let resulting_state = state.add_config(config.clone())
            .unwrap();
        match resulting_state.add_config(config) {
            Err(AppError::Custom(e)) => assert!(e == expected_error),
            Err(_) => panic!("Wrong error received!"),
            _ => panic!("Should not be able to overwrite config!")
        }
    }
    */
}
