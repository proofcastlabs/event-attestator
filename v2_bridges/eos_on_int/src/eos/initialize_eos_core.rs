use common::{traits::DatabaseInterface, types::Result};
use common_eos::maybe_initialize_eos_core_with_eos_account_without_symbol;

pub fn maybe_initialize_eos_core<D: DatabaseInterface>(
    db: &D,
    chain_id: &str,
    eos_account_name: &str,
    eos_init_json: &str,
) -> Result<String> {
    let is_native = true;
    maybe_initialize_eos_core_with_eos_account_without_symbol(db, chain_id, eos_account_name, eos_init_json, is_native)
}
