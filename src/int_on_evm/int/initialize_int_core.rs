use std::fmt;

use crate::{
    chains::eth::{
        core_initialization::{
            check_eth_core_is_initialized::is_eth_core_initialized,
            get_eth_core_init_output_json::EthInitializationOutput,
            initialize_eth_core::initialize_eth_core_with_no_contract_tx,
        },
        eth_chain_id::EthChainId,
        eth_constants::ETH_CORE_IS_INITIALIZED_JSON,
        eth_database_transactions::{
            end_eth_db_transaction_and_return_state,
            start_eth_db_transaction_and_return_state,
        },
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        eth_state::EthState,
        eth_utils::convert_hex_to_eth_address,
    },
    traits::DatabaseInterface,
    types::Result,
};

enum ContractsToAdd {
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

fn add_contract_address_and_return_state<'a, D: DatabaseInterface>(
    state: EthState<'a, D>,
    address_hex: &str,
    contract_to_add: ContractsToAdd,
) -> Result<EthState<'a, D>> {
    info!("✔ Adding {} contract address to db...", contract_to_add);
    convert_hex_to_eth_address(address_hex)
        .and_then(|ref address| match contract_to_add {
            ContractsToAdd::Router => state.eth_db_utils.put_eth_router_smart_contract_address_in_db(address),
            ContractsToAdd::Vault => state.eth_db_utils.put_int_on_evm_smart_contract_address_in_db(address),
        })
        .and(Ok(state))
}

fn add_vault_contract_address_and_return_state<'a, D: DatabaseInterface>(
    state: EthState<'a, D>,
    address_hex: &str,
) -> Result<EthState<'a, D>> {
    add_contract_address_and_return_state(state, address_hex, ContractsToAdd::Vault)
}

fn add_router_contract_address_and_return_state<'a, D: DatabaseInterface>(
    state: EthState<'a, D>,
    address_hex: &str,
) -> Result<EthState<'a, D>> {
    add_contract_address_and_return_state(state, address_hex, ContractsToAdd::Router)
}

/// # Maybe Initialize INT Core
///
/// This function first checks to see if the ETH core has already been initialized, and initializes
/// it if not. The initialization procedure takes as its input a valid ETH block JSON of the
/// format:
///
/// ```no_compile
/// {
///   'block': <eth-block>,
///   'receipts': <block's-receipts>,
/// }
/// ```
/// The function also requires an ETH Chain ID where:
///
/// ```no_compile
/// 1  = Ethereum Mainnet
/// 3  = Ropsten Testnet
/// 4  = Rinkeby Testnet
/// 42 = Kovan Testnet
/// ```
///
/// The function also takes an ETH `gas_price` param, expressed in `Wei`, along with a `canon_to_tip`
/// length param. This latter defines how many `confirmations` of a transactions are required before
/// a signature is signed. Finally, this function requires the addresses of the vault & router
/// smart contracts.
pub fn maybe_initialize_int_core<D: DatabaseInterface>(
    db: D,
    block_json: &str,
    chain_id: u8,
    gas_price: u64,
    confs: u64,
    vault_address: &str,
    router_address: &str,
) -> Result<String> {
    match is_eth_core_initialized(&EthDbUtils::new(&db)) {
        true => Ok(ETH_CORE_IS_INITIALIZED_JSON.to_string()),
        false => start_eth_db_transaction_and_return_state(EthState::init(&db))
            .and_then(|state| {
                initialize_eth_core_with_no_contract_tx(
                    block_json,
                    &EthChainId::try_from(chain_id)?,
                    gas_price,
                    confs,
                    state,
                )
            })
            .and_then(|state| add_vault_contract_address_and_return_state(state, vault_address))
            .and_then(|state| add_router_contract_address_and_return_state(state, router_address))
            .and_then(end_eth_db_transaction_and_return_state)
            .and_then(|state| EthInitializationOutput::new_with_no_contract(&state.eth_db_utils)),
    }
}
