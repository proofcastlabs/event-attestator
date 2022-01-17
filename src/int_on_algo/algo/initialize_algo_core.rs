use crate::{
    chains::{
        algo::{
            algo_constants::ALGO_CORE_IS_INITIALIZED_JSON,
            algo_database_utils::AlgoDbUtils,
            core_initialization::check_algo_core_is_initialized::check_algo_core_is_initialized,
        },
        eth::{
            core_initialization::{
                get_eth_core_init_output_json::EthInitializationOutput,
                initialize_eth_core::initialize_eth_core_with_vault_and_router_contracts_and_return_state,
            },
            eth_chain_id::EthChainId,
            eth_database_transactions::{
                end_eth_db_transaction_and_return_state,
                start_eth_db_transaction_and_return_state,
            },
            eth_state::EthState,
            eth_utils::convert_hex_to_eth_address,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

/// # Maybe Initialize ALGO Core
///
/// This function first checks to see if the ALGO core has already been initialized, and initializes
/// it if not. The initialization procedure takes as its input a valid ALGO block JSON of the
/// format:
///
/// ```no_compile
/// {
///   'block': <eth-block>,
///   'transactions': <block's-transactions>,
/// }
/// ```
pub fn maybe_initialize_algo_core<D: DatabaseInterface>(
    db: D,
    block_json: &str,
    chain_id: u64,
    gas_price: u64,
    confs: u64,
    vault_address: &str,
    router_address: &str,
) -> Result<String> {
    if check_algo_core_is_initialized(&AlgoDbUtils::new(&db)).is_ok() {
        Ok(ALGO_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        Ok("FIXME".to_string()) // FIXME
                                /*
                                start_eth_db_transaction_and_return_state(EthState::init(&db))
                                    .and_then(|state| {
                                        initialize_eth_core_with_vault_and_router_contracts_and_return_state(
                                            block_json,
                                            &EthChainId::try_from(chain_id)?,
                                            gas_price,
                                            confs,
                                            state,
                                            &convert_hex_to_eth_address(vault_address)?,
                                            &convert_hex_to_eth_address(router_address)?,
                                        )
                                    })
                                    .and_then(end_eth_db_transaction_and_return_state)
                                    .and_then(|state| EthInitializationOutput::new_with_no_contract(&state.eth_db_utils)),
                                */
    }
}
