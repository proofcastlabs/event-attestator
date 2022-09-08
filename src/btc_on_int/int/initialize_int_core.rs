use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::{
        core_initialization::{
            get_eth_core_init_output_json::EthInitializationOutput,
            initialize_eth_core::initialize_eth_core_with_no_contract_tx,
        },
        eth_chain_id::EthChainId,
        eth_constants::ETH_CORE_IS_INITIALIZED_JSON,
        eth_database_transactions::{
            end_eth_db_transaction_and_return_state,
            start_eth_db_transaction_and_return_state,
        },
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
        eth_utils::convert_hex_to_eth_address,
    },
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
};

pub fn init_int_core<D: DatabaseInterface>(
    state: EthState<D>,
    block_json: &str,
    chain_id: u64,
    gas_price: u64,
    canon_to_tip_length: u64,
    erc777_contract_address: &EthAddress,
    router_contract_address: &EthAddress,
) -> Result<String> {
    let is_native = false;
    start_eth_db_transaction_and_return_state(state)
        .and_then(|state| {
            initialize_eth_core_with_no_contract_tx(
                block_json,
                &EthChainId::try_from(chain_id)?,
                gas_price,
                canon_to_tip_length,
                state,
                is_native,
            )
        })
        .and_then(|state| {
            state
                .eth_db_utils
                .put_eth_router_smart_contract_address_in_db(router_contract_address)?;
            state
                .eth_db_utils
                .put_btc_on_int_smart_contract_address_in_db(erc777_contract_address)?;
            Ok(state)
        })
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(|state| EthInitializationOutput::new_with_no_contract(&state.eth_db_utils))
}

pub fn maybe_initialize_int_core<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    chain_id: u64,
    gas_price: u64,
    canon_to_tip_length: u64,
    erc777_contract_address: &str,
    router_contract_address: &str,
) -> Result<String> {
    let state = EthState::init(db);
    if CoreType::host_core_is_initialized(db) {
        Ok(ETH_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        init_int_core(
            state,
            block_json,
            chain_id,
            gas_price,
            canon_to_tip_length,
            &convert_hex_to_eth_address(erc777_contract_address)?,
            &convert_hex_to_eth_address(router_contract_address)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        btc_on_int::test_utils::get_sample_int_submission_material_json_str_n,
        chains::eth::{eth_state::EthState, eth_utils::convert_hex_to_eth_address},
        test_utils::get_test_database,
    };

    #[test]
    fn should_init_int_core() {
        let db = get_test_database();
        let eth_block_0 = get_sample_int_submission_material_json_str_n(0);
        let eth_state = EthState::init(&db);
        let eth_chain_id = 3;
        let eth_gas_price = 20_000_000_000;
        let eth_canon_to_tip_length = 2;
        let ptoken_address_hex = "0x0f513aA8d67820787A8FDf285Bfcf967bF8E4B8b";
        let ptoken_address = convert_hex_to_eth_address(ptoken_address_hex).unwrap();
        let router_address_hex = "0x88d19e08cd43bba5761c10c588b2a3d85c75041f";
        let router_address = convert_hex_to_eth_address(router_address_hex).unwrap();
        let result = init_int_core(
            eth_state,
            &eth_block_0,
            eth_chain_id,
            eth_gas_price,
            eth_canon_to_tip_length,
            &ptoken_address,
            &router_address,
        );
        assert!(result.is_ok());
    }
}
