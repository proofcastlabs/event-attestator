use crate::{
    chains::eth::{
        core_initialization::{
            get_eth_core_init_output_json::EthInitializationOutput,
            initialize_eth_core::initialize_evm_core_with_no_contract_tx,
        },
        eth_chain_id::EthChainId,
        eth_constants::EVM_CORE_IS_INITIALIZED_JSON,
        eth_database_transactions::{
            end_eth_db_transaction_and_return_state,
            start_eth_db_transaction_and_return_state,
        },
        eth_state::EthState,
    },
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
};

/// # Maybe Initialize EVM Core
///
/// This function first checks to see if the EVM core has already been initialized, and initializes
/// it if not. The initialization procedure takes as its input a valid EVM-compliant block JSON of the
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
/// 56 = BSC Mainnet
/// ```
/// The function also takes an ETH `gas_price` param, express in `Wei`, along with a `canon_to_tip`
/// length param. This latter defines how many `confirmations` of a transactions are required before
/// a signature is signed.
///
/// ### NOTE:
///
/// The `ERC20-on-EVM` core does NOT require any bytecode passing in since the initialization does NOT
/// return a signed, smart-contract-deploying transaction. This is because the `ERC20-on-EVM` bridge
/// works with an ETH<->EVM token dictionary which defines the contract addresses to be bridged.
pub fn maybe_initialize_evm_core<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    chain_id: u8,
    gas_price: u64,
    confs: u64,
) -> Result<String> {
    if CoreType::host_core_is_initialized(db) {
        Ok(EVM_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        let is_native = false;
        start_eth_db_transaction_and_return_state(EthState::init(db))
            .and_then(|state| {
                initialize_evm_core_with_no_contract_tx(
                    block_json,
                    &EthChainId::try_from(chain_id)?,
                    gas_price,
                    confs,
                    state,
                    is_native,
                )
            })
            .and_then(end_eth_db_transaction_and_return_state)
            .and_then(|state| EthInitializationOutput::new_with_no_contract(&state.evm_db_utils))
    }
}
