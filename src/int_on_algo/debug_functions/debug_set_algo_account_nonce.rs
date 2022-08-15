use serde_json::json;

use crate::{
    chains::{algo::algo_database_utils::AlgoDbUtils, eth::eth_database_utils::EthDbUtils},
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    int_on_algo::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

/// Debug Set Algo Account Nonce
///
/// Sets the Algo account nonce in the database to the passed in value.
pub fn debug_set_algo_account_nonce<D: DatabaseInterface>(
    db: &D,
    nonce: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &AlgoDbUtils::new(db)))
        .and_then(|_| db.start_transaction())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::IntOnAlgo, signature, debug_command_hash))
        .and_then(|_| AlgoDbUtils::new(db).put_algo_account_nonce_in_db(nonce))
        .and_then(|_| db.end_transaction())
        .map(|_| json!({ "algo_account_nonce": nonce }).to_string())
}
