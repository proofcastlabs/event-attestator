use common::{core_type::CoreType, traits::DatabaseInterface, types::Result, EthChainId};
use common_eth::{
    convert_hex_to_eth_address,
    end_eth_db_transaction_and_return_state,
    initialize_eth_core_with_vault_and_router_contracts_and_return_state,
    start_eth_db_transaction_and_return_state,
    EthInitializationOutput,
    EthState,
    VaultUsingCores,
    ETH_CORE_IS_INITIALIZED_JSON,
};

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
    db: &D,
    block_json: &str,
    chain_id: u64,
    gas_price: u64,
    confs: u64,
    vault_address: &str,
    router_address: &str,
) -> Result<String> {
    if CoreType::native_core_is_initialized(db) {
        Ok(ETH_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        let is_native = true;
        start_eth_db_transaction_and_return_state(EthState::init(db))
            .and_then(|state| {
                initialize_eth_core_with_vault_and_router_contracts_and_return_state(
                    block_json,
                    &EthChainId::try_from(chain_id)?,
                    gas_price,
                    confs,
                    state,
                    &convert_hex_to_eth_address(vault_address)?,
                    &convert_hex_to_eth_address(router_address)?,
                    &VaultUsingCores::IntOnEvm,
                    is_native,
                )
            })
            .and_then(end_eth_db_transaction_and_return_state)
            .and_then(|state| EthInitializationOutput::new_with_no_contract(&state.eth_db_utils))
    }
}
