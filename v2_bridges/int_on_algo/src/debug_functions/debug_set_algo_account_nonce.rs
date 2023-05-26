use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_algorand::AlgoDbUtils;
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
use serde_json::json;

use crate::constants::CORE_TYPE;

/// Debug Set Algo Account Nonce
///
/// Sets the Algo account nonce in the database to the passed in value.
#[named]
pub fn debug_set_algo_account_nonce<D: DatabaseInterface>(db: &D, nonce: u64, signature: &str) -> Result<String> {
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), &nonce)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| AlgoDbUtils::new(db).put_algo_account_nonce_in_db(nonce))
        .and_then(|_| db.end_transaction())
        .map(|_| json!({ "algo_account_nonce": nonce }).to_string())
}
