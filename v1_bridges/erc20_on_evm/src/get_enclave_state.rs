use common::{
    chains::eth::{
        eth_database_utils::{EthDbUtils, EthDbUtilsExt, EvmDbUtils},
        eth_enclave_state::{EthEnclaveState, EvmEnclaveState},
    },
    core_type::CoreType,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    traits::DatabaseInterface,
    types::Result,
};
use common_enclave_info::EnclaveInfo;
use serde::{Deserialize, Serialize};

use super::constants::CORE_TYPE;

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    eth: EthEnclaveState,
    evm: EvmEnclaveState,
    token_dictionary: EthEvmTokenDictionary,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, evm_db_utils: &EvmDbUtils<D>) -> Result<Self> {
        let eth_router_address = None;
        Ok(Self {
            info: EnclaveInfo::new(eth_db_utils.get_db())?,
            evm: EvmEnclaveState::new(
                evm_db_utils,
                &evm_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                eth_router_address,
            )?,
            eth: EthEnclaveState::new(
                eth_db_utils,
                &eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                eth_router_address,
            )?,
            token_dictionary: EthEvmTokenDictionary::get_from_db(eth_db_utils.get_db())?,
        })
    }

    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

/// # Get Enclave State
///
/// This function returns a JSON containing the enclave state, including state relevant to each
/// blockchain controlled by this instance.
pub fn get_enclave_state<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting enclave state for {}...", CORE_TYPE);
    CoreType::check_is_initialized(db)
        .and_then(|_| EnclaveState::new(&EthDbUtils::new(db), &EvmDbUtils::new(db))?.to_string())
}
