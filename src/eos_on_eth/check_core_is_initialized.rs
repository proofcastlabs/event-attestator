use crate::{
    chains::{
        eos::{core_initialization::check_eos_core_is_initialized::check_eos_core_is_initialized, eos_state::EosState},
        eth::{
            core_initialization::check_eth_core_is_initialized::check_eth_core_is_initialized,
            eth_database_utils::EthDbUtils,
            eth_state::EthState,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn check_core_is_initialized<D: DatabaseInterface>(eth_db_utils: &EthDbUtils<D>, db: &D) -> Result<()> {
    check_eth_core_is_initialized(eth_db_utils).and_then(|_| check_eos_core_is_initialized(db))
}

pub fn check_core_is_initialized_and_return_eos_state<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    check_core_is_initialized(&state.eth_db_utils, state.db).and(Ok(state))
}

pub fn check_core_is_initialized_and_return_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_core_is_initialized(&state.eth_db_utils, state.db).and(Ok(state))
}
