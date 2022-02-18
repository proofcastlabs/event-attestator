use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::{
        core_initialization::{
            check_eth_core_is_initialized::is_eth_core_initialized as is_int_core_initialized,
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
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn init_int_core<D: DatabaseInterface>(
    state: EthState<D>,
    block_json: &str,
    chain_id: u8,
    gas_price: u64,
    canon_to_tip_length: u64,
    erc777_contract_address: &EthAddress,
    router_contract_address: &EthAddress,
) -> Result<String> {
    start_eth_db_transaction_and_return_state(state)
        .and_then(|state| {
            initialize_eth_core_with_no_contract_tx(
                block_json,
                &EthChainId::try_from(chain_id)?,
                gas_price,
                canon_to_tip_length,
                state,
            )
        })
        .and_then(|state| {
            state.eth_db_utils.put_eth_router_smart_contract_address_in_db(router_contract_address)?;
            state.eth_db_utils.put_btc_on_eth_smart_contract_address_in_db(erc777_contract_address)?;
            Ok(state)
        })
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(|state| EthInitializationOutput::new_with_no_contract(&state.eth_db_utils))
}

pub fn maybe_initialize_int_core<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    chain_id: u8,
    gas_price: u64,
    canon_to_tip_length: u64,
    erc777_contract_address: &EthAddress,
    router_contract_address: &EthAddress,
) -> Result<String> {
    let state = EthState::init(db);
    if is_int_core_initialized(&state.eth_db_utils) {
        Ok(ETH_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        init_int_core(
            state,
            block_json,
            chain_id,
            gas_price,
            canon_to_tip_length,
            erc777_contract_address,
            router_contract_address
        )
    }
}

// TODO Test!
