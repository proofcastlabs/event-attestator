use serde_json::json;

use crate::{
    chains::eth::{
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        eth_utils::get_eth_address_from_str,
    },
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
};

/// # Maybe Add ERC777 Contract Address
///
/// This function will add a passed in ETH address to the encrypted database since the ETH
/// initialization no longer creates one. Until this step has been carried out after ETH core
/// initialization, the `get_enclave_state` command will error with a message telling you to call
/// this function.
///
/// ### BEWARE:
/// The vault contract can only be set ONCE. Further attempts to do so will not succeed.
pub fn maybe_add_vault_contract_address_to_db<D: DatabaseInterface>(db: &D, address: &str) -> Result<String> {
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| {
            EthDbUtils::new(db).put_erc20_on_eos_smart_contract_address_in_db(&get_eth_address_from_str(address)?)
        })
        .and_then(|_| db.end_transaction())
        .map(|_| json!({"success":true, "vaultAddress:": address}).to_string())
}
