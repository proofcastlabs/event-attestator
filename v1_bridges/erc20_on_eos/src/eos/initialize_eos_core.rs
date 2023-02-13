use common::{traits::DatabaseInterface, types::Result};
use common_eos::maybe_initialize_eos_core_without_eos_account_or_symbol;

pub fn maybe_initialize_eos_core<D: DatabaseInterface>(db: &D, chain_id: &str, eos_init_json: &str) -> Result<String> {
    let is_native = false;
    maybe_initialize_eos_core_without_eos_account_or_symbol(db, chain_id, eos_init_json, is_native)
}
