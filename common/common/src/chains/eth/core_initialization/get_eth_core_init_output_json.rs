use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::{
    chains::eth::{eth_database_utils::EthDbUtilsExt, EthState},
    constants::ZERO_CONFS_WARNING,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EthInitializationOutput {
    pub eth_address: String,
    pub eth_latest_block_num: usize,
    pub number_of_confirmations: String,
    pub eth_ptoken_contract_tx: Option<String>,
    pub smart_contract_address: Option<String>,
}

impl EthInitializationOutput {
    fn init<D: DatabaseInterface, E: EthDbUtilsExt<D>>(
        db_utils: &E,
        contract_address: Option<&EthAddress>,
        contract_tx: Option<&str>,
    ) -> Result<Self> {
        let number_of_confirmations = db_utils.get_eth_canon_to_tip_length_from_db()?;
        Ok(Self {
            eth_address: format!(
                "0x{}",
                hex::encode(db_utils.get_public_eth_address_from_db()?.as_bytes())
            ),
            eth_latest_block_num: db_utils.get_latest_eth_block_number()?,
            eth_ptoken_contract_tx: contract_tx.map(|tx| tx.to_string()),
            smart_contract_address: contract_address.map(|address| format!("0x{}", hex::encode(address))),
            number_of_confirmations: if number_of_confirmations == 0 {
                ZERO_CONFS_WARNING.to_string()
            } else {
                number_of_confirmations.to_string()
            },
        })
    }

    pub fn new_with_no_contract<D: DatabaseInterface, E: EthDbUtilsExt<D>>(db_utils: &E) -> Result<String> {
        const CONTRACT_TX: Option<&str> = None;
        const CONTRACT_ADDRESS: Option<&EthAddress> = None;
        Ok(to_string(&Self::init(db_utils, CONTRACT_ADDRESS, CONTRACT_TX)?)?)
    }

    pub fn new_for_eos_on_eth<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
        Self::new_with_no_contract(&state.eth_db_utils)
    }
}
