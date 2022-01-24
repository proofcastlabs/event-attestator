use crate::{
    chains::algo::{
        algo_constants::ALGO_CORE_IS_INITIALIZED_JSON,
        algo_database_transactions::{
            end_algo_db_transaction_and_return_state,
            start_algo_db_transaction_and_return_state,
        },
        algo_database_utils::AlgoDbUtils,
        algo_state::AlgoState,
        core_initialization::{
            check_algo_core_is_initialized::check_algo_core_is_initialized,
            get_algo_core_init_output::AlgoInitializationOutput,
            initialize_algo_core::initialize_algo_core,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

/// # Maybe Initialize ALGO Core
///
/// This function first checks to see if the ALGO core has already been initialized, and initializes
/// it if not. The initialization procedure takes as its input a valid ALGO block JSON of the
/// format:
///
/// ```no_compile
/// {
///   'block': <algo-block>,
/// }
/// ```
pub fn maybe_initialize_algo_core<D: DatabaseInterface>(
    db: D,
    block_json: &str,
    genesis_hash: &str,
    fee: u64,
    confs: u64,
    // FIXME Asset ID? etc
) -> Result<String> {
    if check_algo_core_is_initialized(&AlgoDbUtils::new(&db)).is_ok() {
        Ok(ALGO_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        start_algo_db_transaction_and_return_state(AlgoState::init(&db))
            .and_then(|state| initialize_algo_core(state))// FIXME, block_json, genesis_hash, fee, confs))
            .and_then(end_algo_db_transaction_and_return_state)
            .and_then(|state| AlgoInitializationOutput::new(&state.algo_db_utils))
            .and_then(|output| output.to_string())
    }
}
