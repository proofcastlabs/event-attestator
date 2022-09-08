use crate::{
    chains::eos::core_initialization::initialize_eos_core::maybe_initialize_eos_core_with_eos_account_without_symbol,
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_initialize_eos_core<D: DatabaseInterface>(
    db: &D,
    chain_id: &str,
    eos_account_name: &str,
    eos_init_json: &str,
) -> Result<String> {
    let is_native = true;
    maybe_initialize_eos_core_with_eos_account_without_symbol(db, chain_id, eos_account_name, eos_init_json, is_native)
}
