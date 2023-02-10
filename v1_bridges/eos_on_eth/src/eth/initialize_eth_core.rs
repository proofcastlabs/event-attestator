use common::{core_type::CoreType, traits::DatabaseInterface, types::Result, EthChainId};
use common_eth::{
    end_eth_db_transaction_and_return_state,
    generate_and_store_eos_on_eth_contract_address,
    initialize_eth_core_with_no_contract_tx,
    start_eth_db_transaction_and_return_state,
    EthInitializationOutput,
    EthState,
    ETH_CORE_IS_INITIALIZED_JSON,
};

/// # Maybe Initialize ETH Core
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
/// The function also takes an ETH `gas_price` param, express in `Wei`, along with a `canon_to_tip`
/// length param. This latter defines how many `confirmations` of a transactions are required before
/// a signature is signed.
///
/// ### NOTE:
///
/// The `eos-on-eth` core does NOT require any bytecode passing in since the initialization does NOT
/// return a signed, smart-contract-deploying transaction. This is because the `eos-on-eth` bridge
/// works with an EOS<->ETH token dictionary which defines the contract addresses to be bridged.
pub fn maybe_initialize_eth_core<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    chain_id: u8,
    gas_price: u64,
    confs: u64,
) -> Result<String> {
    if CoreType::host_core_is_initialized(db) {
        Ok(ETH_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        let is_native = false;
        start_eth_db_transaction_and_return_state(EthState::init(db))
            .and_then(|state| {
                initialize_eth_core_with_no_contract_tx(
                    block_json,
                    &EthChainId::try_from(chain_id)?,
                    gas_price,
                    confs,
                    state,
                    is_native,
                )
            })
            .and_then(generate_and_store_eos_on_eth_contract_address)
            .and_then(end_eth_db_transaction_and_return_state)
            .and_then(EthInitializationOutput::new_for_eos_on_eth)
    }
}
