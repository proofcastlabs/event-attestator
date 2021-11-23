use crate::{
    chains::eth::{eth_database_utils::EthDbUtilsExt, eth_state::EthState},
    traits::DatabaseInterface,
    types::Result,
};

fn check_for_parent_of_block_in_state<D: DatabaseInterface>(
    is_for_eth: bool,
    state: EthState<D>,
) -> Result<EthState<D>> {
    let block_type = if is_for_eth { "ETH" } else { "EVM" };
    info!("✔ Checking if {} block's parent exists in database...", block_type);
    let parent_hash = state.get_parent_hash()?;
    let parent_exists = if is_for_eth {
        state.eth_db_utils.eth_block_exists_in_db(&parent_hash)
    } else {
        state.evm_db_utils.eth_block_exists_in_db(&parent_hash)
    };
    if parent_exists {
        info!("✔ {} block's parent exists in database!", block_type);
        Ok(state)
    } else {
        Err(format!("✘ {} block Rejected - no parent exists in database!", block_type).into())
    }
}

pub fn check_for_parent_of_eth_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_for_parent_of_block_in_state(true, state)
}

pub fn check_for_parent_of_evm_block_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_for_parent_of_block_in_state(false, state)
}
