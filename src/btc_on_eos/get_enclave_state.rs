use serde::{Deserialize, Serialize};

use crate::{
    btc_on_eos::check_core_is_initialized::check_core_is_initialized,
    chains::{
        btc::{btc_database_utils::BtcDbUtils, btc_enclave_state::BtcEnclaveState},
        eos::{eos_database_utils::EosDbUtils, eos_enclave_state::EosEnclaveState},
    },
    enclave_info::EnclaveInfo,
    fees::fee_enclave_state::FeesEnclaveState,
    traits::DatabaseInterface,
    types::Result,
};

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
pub fn get_enclave_state<D: DatabaseInterface>(db: D) -> Result<String> {
    info!("✔ Getting core state...");
    let btc_db_utils = BtcDbUtils::new(&db);
    let eos_db_utils = EosDbUtils::new(&db);
    check_core_is_initialized(&btc_db_utils, &eos_db_utils)
        .and_then(|_| EnclaveState::new(&btc_db_utils, &eos_db_utils)?.to_string())
}
