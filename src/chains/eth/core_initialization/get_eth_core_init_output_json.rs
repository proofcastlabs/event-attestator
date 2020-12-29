use crate::{
    chains::eth::{
        eth_database_utils::{
            get_eos_on_eth_smart_contract_address_from_db,
            get_erc20_on_eos_smart_contract_address_from_db,
            get_erc777_contract_address_from_db,
            get_latest_eth_block_number,
            get_public_eth_address_from_db,
        },
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};
use ethereum_types::Address as EthAddress;
use serde_json::to_string;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EthInitializationOutput {
    pub eth_address: String,
    pub eth_latest_block_num: usize,
    pub eth_ptoken_contract_tx: String,
    pub smart_contract_address: String,
}

impl EthInitializationOutput {
    fn init<D: DatabaseInterface>(db: &D, contract_address: &EthAddress, contract_tx: &str) -> Result<Self> {
        Ok(Self {
            eth_address: format!("0x{}", hex::encode(get_public_eth_address_from_db(db)?.as_bytes())),
            eth_latest_block_num: get_latest_eth_block_number(db)?,
            eth_ptoken_contract_tx: contract_tx.to_string(),
            smart_contract_address: format!("0x{}", hex::encode(contract_address)),
        })
    }

    pub fn new_for_eos_on_eth<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
        Ok(to_string(&Self::init(
            &state.db,
            &get_eos_on_eth_smart_contract_address_from_db(&state.db)?,
            &state.get_misc_string()?,
        )?)?)
    }

    pub fn new_for_btc_on_eth<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
        Ok(to_string(&Self::init(
            &state.db,
            &get_erc777_contract_address_from_db(&state.db)?,
            &state.get_misc_string()?,
        )?)?)
    }

    pub fn new_for_erc20_on_eth<D: DatabaseInterface>(state: EthState<D>) -> Result<String> {
        Ok(to_string(&Self::init(
            &state.db,
            &get_erc20_on_eos_smart_contract_address_from_db(&state.db)?,
            &state.get_misc_string()?,
        )?)?)
    }
}
