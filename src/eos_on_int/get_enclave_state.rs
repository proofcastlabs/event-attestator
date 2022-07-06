use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::{
    chains::{
        eos::{eos_database_utils::EosDbUtils, eos_enclave_state::EosEnclaveState},
        eth::{
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
            eth_enclave_state::EthEnclaveState,
        },
    },
    enclave_info::EnclaveInfo,
    eos_on_eth::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    eos: EosEnclaveState,
    eth: EthEnclaveState,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, eos_db_utils: &EosDbUtils<D>) -> Result<Self> {
        Ok(Self {
            info: EnclaveInfo::new(),
            eos: EosEnclaveState::new(eos_db_utils)?,
            eth: EthEnclaveState::new(
                eth_db_utils,
                &EthAddress::zero(),
                Some(eth_db_utils.get_eth_router_smart_contract_address_from_db()?),
            )?,
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
    info!("✔ Getting core state...");
    let eth_db_utils = EthDbUtils::new(&db);
    let eos_db_utils = EosDbUtils::new(&db);
    check_core_is_initialized(&eth_db_utils, &eos_db_utils)
        .and_then(|_| EnclaveState::new(&eth_db_utils, &eos_db_utils)?.to_string())
}
