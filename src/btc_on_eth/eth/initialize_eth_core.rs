use crate::{
    chains::eth::{
        core_initialization::{
            check_eth_core_is_initialized::is_eth_core_initialized,
            eth_core_init_utils::check_for_existence_of_eth_contract_byte_code,
            generate_eth_contract_address::generate_and_store_btc_on_eth_contract_address,
            get_eth_core_init_output_json::EthInitializationOutput,
            initialize_eth_core::initialize_eth_core,
        },
        eth_constants::ETH_CORE_IS_INITIALIZED_JSON,
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn maybe_initialize_eth_enclave<D: DatabaseInterface>(
    db: D,
    block_json: &str,
    chain_id: u8,
    gas_price: u64,
    canon_to_tip_length: u64,
    bytecode_path: &str,
) -> Result<String> {
    check_for_existence_of_eth_contract_byte_code(bytecode_path).and_then(|_| match is_eth_core_initialized(&db) {
        true => Ok(ETH_CORE_IS_INITIALIZED_JSON.to_string()),
        false => initialize_eth_core(
            block_json,
            chain_id,
            gas_price,
            canon_to_tip_length,
            bytecode_path,
            EthState::init(db),
        )
        .and_then(generate_and_store_btc_on_eth_contract_address)
        .and_then(EthInitializationOutput::new_for_btc_on_eth),
    })
}
