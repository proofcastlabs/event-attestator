use serde::{Deserialize, Serialize};

use crate::{
    chains::eth::{
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        eth_enclave_state::EthEnclaveState,
    },
    dictionaries::eth_evm::EthEvmTokenDictionary,
    enclave_info::EnclaveInfo,
    erc20_on_evm::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    eth: EthEnclaveState,
    evm: EthEnclaveState,
    token_dictionary: EthEvmTokenDictionary,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, evm_db_utils: &EthDbUtils<D>) -> Result<Self> {
        Ok(Self {
            info: EnclaveInfo::new(),
            evm: EthEnclaveState::new_for_erc20_on_evm(evm_db_utils)?,
            eth: EthEnclaveState::new_for_erc20_on_evm(eth_db_utils)?,
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
    let eth_db_utils = EthDbUtils::new_for_eth(&db);
    let evm_db_utils = EthDbUtils::new_for_evm(&db);
    check_core_is_initialized(&eth_db_utils, &evm_db_utils)
        .and_then(|_| EnclaveState::new(&eth_db_utils, &evm_db_utils)?.to_string())
}
