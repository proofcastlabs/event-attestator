use std::fmt;

use crate::{
    chains::eth::{
        eth_database_utils::{EthDbUtils, EthDbUtilsExt, EvmDbUtils},
        eth_utils::convert_hex_to_eth_address,
    },
    erc20_on_int::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

pub enum ContractsToAdd {
    Router,
    Vault,
}

impl fmt::Display for ContractsToAdd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Vault => write!(f, "vault"),
            Self::Router => write!(f, "router"),
        }
    }
}

pub fn maybe_add_contract_address<D: DatabaseInterface>(
    db: D,
    address_hex: &str,
    contract_to_add: ContractsToAdd,
) -> Result<String> {
    let eth_db_utils = EthDbUtils::new(&db);
    let evm_db_utils = EvmDbUtils::new(&db);
    info!("✔ Adding {} contract address to db...", contract_to_add);
    check_core_is_initialized(&eth_db_utils, &evm_db_utils)
        .and_then(|_| db.start_transaction())
        .and_then(|_| convert_hex_to_eth_address(address_hex))
        .and_then(|ref address| match contract_to_add {
            ContractsToAdd::Router => eth_db_utils.put_eth_router_smart_contract_address_in_db(address),
            ContractsToAdd::Vault => eth_db_utils.put_erc20_on_evm_smart_contract_address_in_db(address),
        })
        .and_then(|_| db.end_transaction())
        .map(|_| format!("{{add_{}_address_success:true}}", contract_to_add))
}

/// # Maybe Add Vault Contract Address
///
/// This function will add a passed in ETH contract address to the encrypted database since the ETH
/// initialization no longer creates one. Until this step has been carried out after ETH core
/// initialization, the `get_enclave_state` command will error with a message telling you to call
/// this function.
///
/// ### BEWARE:
/// This vault contract setter can only be set ONCE. Further attempts to do so will not succeed.
pub fn maybe_add_vault_contract_address<D: DatabaseInterface>(db: D, address_hex: &str) -> Result<String> {
    maybe_add_contract_address(db, address_hex, ContractsToAdd::Vault)
}

/// # Maybe Add Router Contract Address
///
/// This function will add a passed in ETH contract address to the encrypted database Until this
/// step has been carried out after ETH core initialization, the `get_enclave_state` command will
/// error with a message telling you to call this function.
///
/// ### BEWARE:
/// This vault contract setter can only be set ONCE. Further attempts to do so will not succeed.
pub fn maybe_add_router_contract_address<D: DatabaseInterface>(db: D, address_hex: &str) -> Result<String> {
    maybe_add_contract_address(db, address_hex, ContractsToAdd::Router)
}
