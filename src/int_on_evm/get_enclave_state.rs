use serde::{Deserialize, Serialize};

use crate::{
    chains::eth::{
        eth_database_utils::{EthDbUtils, EthDbUtilsExt, EvmDbUtils},
        eth_enclave_state::{EthEnclaveState, EvmEnclaveState},
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    enclave_info::EnclaveInfo,
    int_on_evm::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    int: EthEnclaveState,
    evm: EvmEnclaveState,
    token_dictionary: EthEvmTokenDictionary,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, evm_db_utils: &EvmDbUtils<D>) -> Result<Self> {
        Ok(Self {
            info: EnclaveInfo::new(eth_db_utils.get_db())?,
            evm: EvmEnclaveState::new(
                evm_db_utils,
                &evm_db_utils.get_int_on_evm_smart_contract_address_from_db()?,
                Some(evm_db_utils.get_eth_router_smart_contract_address_from_db()?),
            )?,
            int: EthEnclaveState::new(
                eth_db_utils,
                &eth_db_utils.get_int_on_evm_smart_contract_address_from_db()?,
                Some(eth_db_utils.get_eth_router_smart_contract_address_from_db()?),
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
pub fn get_enclave_state<D: DatabaseInterface>(db: D) -> Result<String> {
    info!("✔ Getting enclave state...");
    let eth_db_utils = EthDbUtils::new(&db);
    let evm_db_utils = EvmDbUtils::new(&db);
    check_core_is_initialized(&eth_db_utils, &evm_db_utils)
        .and_then(|_| EnclaveState::new(&eth_db_utils, &evm_db_utils)?.to_string())
}
