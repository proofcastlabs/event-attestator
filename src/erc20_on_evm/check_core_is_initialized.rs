use crate::{
    chains::{
        eth::{
            core_initialization::check_eth_core_is_initialized::is_eth_core_initialized,
            eth_database_utils::EthDatabaseUtils,
            eth_state::EthState,
        },
        evm::core_initialization::check_eth_core_is_initialized::is_eth_core_initialized as is_evm_core_initialized,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn check_core_is_initialized<D: DatabaseInterface>(eth_db_utils: &EthDatabaseUtils<D>, db: &D) -> Result<()> {
    info!("✔ Checking `erc20-on-evm` core is initialized...");
    match is_evm_core_initialized(db) {
        false => Err("EVM core not initialized!".into()),
        true => match is_eth_core_initialized(eth_db_utils) {
            false => Err("ETH core not initialized!".into()),
            true => Ok(()),
        },
    }
}

pub fn check_core_is_initialized_and_return_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_core_is_initialized(&state.eth_db_utils, state.db).and(Ok(state))
}
