use crate::{
    chains::eth::{
        core_initialization::check_eth_core_is_initialized::is_eth_core_initialized,
        eth_database_utils::{EthDbUtils, EvmDbUtils},
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn check_core_is_initialized<D: DatabaseInterface>(
    eth_db_utils: &EthDbUtils<D>,
    evm_db_utils: &EvmDbUtils<D>,
) -> Result<()> {
    info!("✔ Checking `erc20-on-evm` core is initialized...");
    match is_eth_core_initialized(evm_db_utils) {
        false => Err("EVM core not initialized!".into()),
        true => match is_eth_core_initialized(eth_db_utils) {
            false => Err("ETH core not initialized!".into()),
            true => Ok(()),
        },
    }
}

pub fn check_core_is_initialized_and_return_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_core_is_initialized(&state.eth_db_utils, &state.evm_db_utils).and(Ok(state))
}
