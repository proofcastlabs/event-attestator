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

pub fn maybe_initialize_eth_enclave<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    chain_id: u8,
    gas_price: u64,
    confs: u64,
    erc777_contract_address: &str,
) -> Result<String> {
    if CoreType::host_core_is_initialized(db) {
        Ok(ETH_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        start_eth_db_transaction_and_return_state(EthState::init(db))
            .and_then(|state| {
                initialize_eth_core_with_no_contract_tx(
                    block_json,
                    &EthChainId::try_from(chain_id)?,
                    gas_price,
                    confs,
                    state,
                )
            })
            .and_then(|state| {
                state
                    .eth_db_utils
                    .put_btc_on_int_smart_contract_address_in_db(&convert_hex_to_eth_address(
                        erc777_contract_address,
                    )?)?;
                Ok(state)
            })
            .and_then(end_eth_db_transaction_and_return_state)
            .and_then(|state| EthInitializationOutput::new_with_no_contract(&state.eth_db_utils))
    }
}
