use crate::btc_on_eos::{
    types::Result,
    errors::AppError,
    traits::DatabaseInterface,
    eos::eos_types::EosSignedTransactions,
    utils::{
        get_not_in_state_err,
        get_no_overwrite_state_err,
    },
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EosState<D: DatabaseInterface> {
    pub db: D,
    pub eos_signed_txs: Option<EosSignedTransactions>,
}

impl<D> EosState<D> where D: DatabaseInterface {
    pub fn init(db: D) -> Result<EosState<D>> {
        Ok(
            EosState {
                db,
                eos_signed_txs: None,
            }
        )
    }

    pub fn add_eos_signed_txs(
        mut self,
        eos_signed_txs: EosSignedTransactions,
    ) -> Result<EosState<D>>
        where D: DatabaseInterface
    {
        match self.eos_signed_txs {
            Some(_) => Err(AppError::Custom(
                get_no_overwrite_state_err("eos_signed_txs"))
            ),
            None => {
                self.eos_signed_txs = Some(eos_signed_txs);
                Ok(self)
            }
        }
    }

    /*
    pub fn add_eos_block_and_action( // TODO Rename to block and action proofs
        mut self,
        eos_block_and_action: EosBlockAndAction
    ) -> Result<EosState> {
        match self.eos_block_and_action {
            Some(_) => Err(AppError::Custom(
                get_no_overwrite_state_err("eos_block_and_action"))
            ),
            None => {
                self.eos_block_and_action = Some(eos_block_and_action);
                Ok(self)
            }
        }
    }

    pub fn get_eos_block_and_action(
        &self
    ) -> Result<&EosBlockAndAction> {
        match &self.eos_block_and_action{
            Some(eos_block_and_action) => Ok(&eos_block_and_action),
            None => Err(AppError::Custom(
                get_not_in_state_err("eos_block_and_action"))
            )
        }
    }
    */

    pub fn get_eos_signed_txs(&self) -> Result<&EosSignedTransactions> {
        match &self.eos_signed_txs {
            Some(eos_signed_txs) => Ok(&eos_signed_txs),
            None => Err(AppError::Custom(
                get_not_in_state_err("eos_signed_txs"))
            )
        }
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
