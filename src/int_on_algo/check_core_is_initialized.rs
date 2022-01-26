use crate::{
    chains::{
        algo::{
            algo_database_utils::AlgoDbUtils,
            algo_state::AlgoState,
            core_initialization::check_algo_core_is_initialized::check_algo_core_is_initialized,
        },
        eth::{
            core_initialization::check_eth_core_is_initialized::check_eth_core_is_initialized,
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
            eth_state::EthState,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn check_core_is_initialized<D: DatabaseInterface>(
    eth_db_utils: &EthDbUtils<D>,
    algo_db_utils: &AlgoDbUtils<D>,
) -> Result<()> {
    check_algo_core_is_initialized(algo_db_utils).and_then(|_| check_eth_core_is_initialized(eth_db_utils))
}

pub fn check_core_is_initialized_and_return_algo_state<D: DatabaseInterface>(
    state: AlgoState<D>,
) -> Result<AlgoState<D>> {
    check_core_is_initialized(&state.eth_db_utils, &state.algo_db_utils).and(Ok(state))
}

pub fn check_core_is_initialized_and_return_eth_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    check_core_is_initialized(&state.eth_db_utils, &state.algo_db_utils).and(Ok(state))
}
