use crate::{
    chains::{eos::eos_enclave_state::EosEnclaveState, eth::eth_enclave_state::EthEnclaveState},
    enclave_info::EnclaveInfo,
    erc20_on_eos::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
struct EnclaveState {
    info: EnclaveInfo,
    eth: EthEnclaveState,
    eos: EosEnclaveState,
}

pub fn get_enclave_state<D: DatabaseInterface>(db: D) -> Result<String> {
    info!("✔ Getting enclave state...");
    check_core_is_initialized(&db).and_then(|_| {
        Ok(serde_json::to_string(&EnclaveState {
            info: EnclaveInfo::new(),
            eth: EthEnclaveState::new(&db)?,
            eos: EosEnclaveState::new(&db)?,
        })?)
    })
}
