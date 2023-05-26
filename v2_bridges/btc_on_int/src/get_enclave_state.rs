use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_btc::{BtcDbUtils, BtcEnclaveState};
use common_enclave_info::EnclaveInfo;
use common_eth::{EthDbUtils, EthDbUtilsExt, EthEnclaveState};
use serde::{Deserialize, Serialize};

use crate::constants::CORE_TYPE;

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    btc: BtcEnclaveState,
    eth: EthEnclaveState,
}

impl EnclaveState {
    pub fn new<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, btc_db_utils: &BtcDbUtils<D>) -> Result<Self> {
        Ok(Self {
            info: EnclaveInfo::new(eth_db_utils.get_db())?,
            btc: BtcEnclaveState::new(btc_db_utils.get_db(), btc_db_utils)?,
            eth: EthEnclaveState::new(
                eth_db_utils,
                &eth_db_utils.get_btc_on_int_smart_contract_address_from_db()?,
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
pub fn get_enclave_state<D: DatabaseInterface>(db: &D) -> Result<String> {
    info!("✔ Getting enclave state for {} common...", CORE_TYPE);
    CoreType::check_is_initialized(db)
        .and_then(|_| EnclaveState::new(&EthDbUtils::new(db), &BtcDbUtils::new(db))?.to_string())
}
