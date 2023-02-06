use common::{
    chains::{
        btc::{btc_database_utils::BtcDbUtils, btc_enclave_state::BtcEnclaveState},
        eos::{eos_database_utils::EosDbUtils, eos_enclave_state::EosEnclaveState},
    },
    core_type::CoreType,
    enclave_info::EnclaveInfo,
    fees::fee_enclave_state::FeesEnclaveState,
    traits::DatabaseInterface,
    types::Result,
};
use serde::{Deserialize, Serialize};

use crate::constants::CORE_TYPE;

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    eos: EosEnclaveState,
    btc: BtcEnclaveState,
    fees: FeesEnclaveState,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(btc_db_utils: &BtcDbUtils<D>, eos_db_utils: &EosDbUtils<D>) -> Result<Self> {
        let db = btc_db_utils.get_db();
        Ok(Self {
            info: EnclaveInfo::new(db)?,
            eos: EosEnclaveState::new(eos_db_utils)?,
            btc: BtcEnclaveState::new(db, btc_db_utils)?,
            fees: FeesEnclaveState::new_for_btc_on_eos(db)?,
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
    info!("✔ Getting {} core state...", CORE_TYPE);
    CoreType::check_is_initialized(db)
        .and_then(|_| EnclaveState::new(&BtcDbUtils::new(db), &EosDbUtils::new(db))?.to_string())
}
