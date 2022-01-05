use crate::{
    chains::eth::{eth_chain_id::EthChainId, eth_database_utils::EthDbUtilsExt, eth_state::EthState},
    constants::{CORE_IS_VALIDATING, DEBUG_MODE, NOT_VALIDATING_WHEN_NOT_IN_DEBUG_MODE_ERROR},
    traits::DatabaseInterface,
    types::Result,
};

pub fn validate_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if CORE_IS_VALIDATING {
        let chain_id = state.eth_db_utils.get_eth_chain_id_from_db()?;
        if chain_id == EthChainId::Rinkeby {
            // FIXME/TODO Add in rinkeby block validation!
            info!("✔ Skipping Rinkeby ETH block header validaton!");
            Ok(state)
        } else {
            info!("✔ Validating block header...");
            match state
                .get_eth_submission_material()?
                .get_block()?
                .is_valid(&state.eth_db_utils.get_eth_chain_id_from_db()?)?
            {
                true => Ok(state),
                false => Err("✘ Not accepting ETH block - header hash not valid!".into()),
            }
        }
    } else {
        info!("✔ Skipping ETH block header validaton!");
        match DEBUG_MODE {
            true => Ok(state),
            false => Err(NOT_VALIDATING_WHEN_NOT_IN_DEBUG_MODE_ERROR.into()),
        }
    }
}
