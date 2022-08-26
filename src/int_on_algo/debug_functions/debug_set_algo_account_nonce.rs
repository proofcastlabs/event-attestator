use function_name::named;
use serde_json::json;

use crate::{
    chains::{algo::algo_database_utils::AlgoDbUtils, eth::eth_database_utils::EthDbUtils},
    debug_functions::validate_debug_command_signature,
    int_on_algo::{check_core_is_initialized::check_core_is_initialized, constants::CORE_TYPE},
    traits::DatabaseInterface,
    types::Result,
};

/// Debug Set Algo Account Nonce
///
/// Sets the Algo account nonce in the database to the passed in value.
#[named]
pub fn debug_set_algo_account_nonce<D: DatabaseInterface>(db: &D, nonce: u64, signature: &str) -> Result<String> {
    db.start_transaction()
        .and_then(|_| check_core_is_initialized(&EthDbUtils::new(db), &AlgoDbUtils::new(db)))
        .and_then(|_| get_debug_command_hash!(function_name!(), &nonce)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| AlgoDbUtils::new(db).put_algo_account_nonce_in_db(nonce))
        .and_then(|_| db.end_transaction())
        .map(|_| json!({ "algo_account_nonce": nonce }).to_string())
}
