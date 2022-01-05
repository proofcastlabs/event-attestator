use serde::{Deserialize, Serialize};

use crate::{
    btc_on_eth::check_core_is_initialized::check_core_is_initialized,
    chains::{
        btc::btc_enclave_state::BtcEnclaveState,
        eth::{
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
            eth_enclave_state::EthEnclaveState,
        },
    },
    enclave_info::EnclaveInfo,
    fees::fee_enclave_state::FeesEnclaveState,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    btc: BtcEnclaveState,
    eth: EthEnclaveState,
    fees: FeesEnclaveState,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, db: &D) -> Result<Self> {
        Ok(Self {
            info: EnclaveInfo::new(),
            btc: BtcEnclaveState::new(db)?,
            eth: EthEnclaveState::new(eth_db_utils, &eth_db_utils.get_erc777_contract_address_from_db()?, None)?,
            fees: FeesEnclaveState::new_for_btc_on_eth(db)?,
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
    check_core_is_initialized(&eth_db_utils, &db).and_then(|_| EnclaveState::new(&eth_db_utils, &db)?.to_string())
}
