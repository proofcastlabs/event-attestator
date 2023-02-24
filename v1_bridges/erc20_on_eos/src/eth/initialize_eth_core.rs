use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_chain_ids::EthChainId;
use common_eth::{
    convert_hex_to_eth_address,
    end_eth_db_transaction_and_return_state,
    initialize_eth_core_with_no_contract_tx,
    start_eth_db_transaction_and_return_state,
    EthDbUtilsExt,
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
///
/// The function also takes an ETH `gas_price` param, express in `Wei`, along with a `canon_to_tip`
/// length param. This latter defines how many `confirmations` of a transactions are required before
/// a signature is signed.
pub fn maybe_initialize_eth_core<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    chain_id: u8,
    gas_price: u64,
    confs: u64,
    vault_address: &str,
) -> Result<String> {
    if CoreType::native_core_is_initialized(db) {
        Ok(ETH_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        let is_native = true;
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
            .and_then(|state| {
                state
                    .eth_db_utils
                    .put_erc20_on_eos_smart_contract_address_in_db(&convert_hex_to_eth_address(vault_address)?)?;
                Ok(state)
            })
            .and_then(end_eth_db_transaction_and_return_state)
            .and_then(|state| EthInitializationOutput::new_with_no_contract(&state.eth_db_utils))
    }
}
