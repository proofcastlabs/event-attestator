use common::{
    chains::{
        eos::{eos_database_utils::EosDbUtils, eos_enclave_state::EosEnclaveState},
        eth::{
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
            eth_enclave_state::EthEnclaveState,
        },
    },
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
};
use common_enclave_info::EnclaveInfo;
use serde::{Deserialize, Serialize};

use super::constants::CORE_TYPE;

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    int: EthEnclaveState,
    eos: EosEnclaveState,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, eos_db_utils: &EosDbUtils<D>) -> Result<Self> {
        Ok(Self {
            info: EnclaveInfo::new(eth_db_utils.get_db())?,
            int: EthEnclaveState::new(
                eth_db_utils,
                &eth_db_utils.get_int_on_eos_smart_contract_address_from_db()?,
                Some(eth_db_utils.get_eth_router_smart_contract_address_from_db()?),
            )?,
            eos: EosEnclaveState::new_without_account_name(eos_db_utils)?,
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
        .and_then(|_| EnclaveState::new(&EthDbUtils::new(db), &EosDbUtils::new(db))?.to_string())
}
