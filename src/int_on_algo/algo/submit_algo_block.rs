use crate::{
    chains::algo::{
        add_latest_algo_block::add_latest_algo_block_and_return_state,
        algo_database_transactions::start_algo_db_transaction_and_return_state,
        algo_state::AlgoState,
        algo_submission_material::parse_algo_submission_material_and_put_in_state,
        check_parent_exists::check_parent_of_algo_block_in_state_exists,
        remove_old_algo_tail_block::maybe_remove_old_algo_tail_block_and_return_state,
        update_algo_canon_block_hash::maybe_update_algo_canon_block_hash_and_return_state,
        update_algo_linker_hash::maybe_update_algo_linker_hash_and_return_state,
        update_algo_tail_block_hash::maybe_update_algo_tail_block_hash_and_return_state,
    },
    int_on_algo::check_core_is_initialized::check_core_is_initialized_and_return_algo_state,
    traits::DatabaseInterface,
    types::Result,
};

/// Submit Algo Block To Core
///
/// The main submission pipeline. Submitting an Algorand block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ALGO
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain pertinent transactions to the redeem addres  the enclave is watching, an INT
/// transaction will be signed & returned to the caller.
pub fn submit_algo_block_to_core<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting ALGO block to core...");
    parse_algo_submission_material_and_put_in_state(block_json_string, AlgoState::init(&db))
        .and_then(check_core_is_initialized_and_return_algo_state)
        .and_then(start_algo_db_transaction_and_return_state)
        .and_then(check_parent_of_algo_block_in_state_exists)
        .and_then(add_latest_algo_block_and_return_state)
        .and_then(maybe_update_algo_canon_block_hash_and_return_state)
        .and_then(maybe_update_algo_tail_block_hash_and_return_state)
        .and_then(maybe_update_algo_linker_hash_and_return_state)
        .and_then(maybe_remove_old_algo_tail_block_and_return_state)
        .map(|_| "done!".to_string())
}
