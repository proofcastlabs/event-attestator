use common::{traits::DatabaseInterface, types::Result};
use common_eos::maybe_initialize_eos_core_with_eos_account_and_symbol;

pub fn maybe_initialize_eos_core<D: DatabaseInterface>(
    db: &D,
    chain_id: &str,
    account_name: &str,
    token_symbol: &str,
    eos_init_json: &str,
) -> Result<String> {
    let is_native = false;
    maybe_initialize_eos_core_with_eos_account_and_symbol(
        db,
        chain_id,
        account_name,
        token_symbol,
        eos_init_json,
        is_native,
    )
}
