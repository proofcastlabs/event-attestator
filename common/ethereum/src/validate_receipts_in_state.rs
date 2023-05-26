use common::{constants::CORE_IS_VALIDATING, traits::DatabaseInterface, types::Result};

use crate::EthState;

pub fn validate_receipts_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if !CORE_IS_VALIDATING {
        info!("✔ Skipping ETH receipts validation!");
        Ok(state)
    } else {
        info!("✔ Validating receipts...");
        if state.get_eth_submission_material()?.receipts_are_valid()? {
            info!("✔ Receipts are valid!");
            Ok(state)
        } else {
            Err("✘ Not accepting ETH block - receipts root not valid!".into())
        }
    }
}
