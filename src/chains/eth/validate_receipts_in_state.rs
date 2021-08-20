use crate::{
    chains::eth::eth_state::EthState,
    constants::{CORE_IS_VALIDATING, DEBUG_MODE, NOT_VALIDATING_WHEN_NOT_IN_DEBUG_MODE_ERROR},
    traits::DatabaseInterface,
    types::Result,
};

pub fn validate_receipts_in_state<D>(state: EthState<D>) -> Result<EthState<D>>
where
    D: DatabaseInterface,
{
    if CORE_IS_VALIDATING {
        info!("✔ Validating receipts...");
        match state.get_eth_submission_material()?.receipts_are_valid()? {
            true => {
                info!("✔ Receipts are valid!");
                Ok(state)
            },
            false => Err("✘ Not accepting ETH block - receipts root not valid!".into()),
        }
    } else {
        info!("✔ Skipping ETH receipts validation!");
        match DEBUG_MODE {
            true => Ok(state),
            false => Err(NOT_VALIDATING_WHEN_NOT_IN_DEBUG_MODE_ERROR.into()),
        }
    }
}
