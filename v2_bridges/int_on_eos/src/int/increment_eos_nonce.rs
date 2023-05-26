use common::{traits::DatabaseInterface, types::Result};
use common_eos::{increment_eos_account_nonce, EosDbUtils};
use common_eth::EthState;

use super::IntOnEosEosTxInfos;

pub fn maybe_increment_eos_account_nonce_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if state.tx_infos.is_empty() {
        info!("✔ No signatures in state ∴ not incrementing eos account nonce");
        Ok(state)
    } else {
        let eos_db_utils = EosDbUtils::new(state.db);
        increment_eos_account_nonce(
            &eos_db_utils,
            eos_db_utils.get_eos_account_nonce_from_db()?,
            IntOnEosEosTxInfos::from_bytes(&state.tx_infos)?.len() as u64,
        )
        .and(Ok(state))
    }
}
