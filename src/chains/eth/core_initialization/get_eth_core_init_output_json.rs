use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};
use serde_json::to_string;

use crate::{
    chains::eth::{
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EthInitializationOutput {
    pub eth_address: String,
    pub eth_latest_block_num: usize,
    pub eth_ptoken_contract_tx: Option<String>,
    pub smart_contract_address: Option<String>,
}

impl EthInitializationOutput {
    fn init<D: DatabaseInterface>(
        eth_db_utils: &EthDbUtils<D>,
        contract_address: Option<&EthAddress>,
        contract_tx: Option<&str>,
    ) -> Result<Self> {
        Ok(Self {
            eth_address: format!(
                "0x{}",
                hex::encode(eth_db_utils.get_public_eth_address_from_db()?.as_bytes())
            ),
            eth_latest_block_num: eth_db_utils.get_latest_eth_block_number()?,
            eth_ptoken_contract_tx: contract_tx.map(|tx| tx.to_string()),
            smart_contract_address: contract_address.map(|address| format!("0x{}", hex::encode(address))),
        })
    }

    pub fn new_with_no_contract<D: DatabaseInterface>(db_utils: &EthDbUtils<D>) -> Result<String> {
        const CONTRACT_TX: Option<&str> = None;
        const CONTRACT_ADDRESS: Option<&EthAddress> = None;
        Ok(to_string(&Self::init(db_utils, CONTRACT_ADDRESS, CONTRACT_TX)?)?)
    }

    pub fn new_for_eos_on_eth<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
        Self::new_with_no_contract(&state.eth_db_utils)
    }
}
