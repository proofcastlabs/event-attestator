use crate::{
    chains::eth::{
        core_initialization::{
            check_eth_core_is_initialized::is_eth_core_initialized,
            eth_core_init_utils::check_for_existence_of_eth_contract_byte_code,
            generate_eth_contract_address::generate_and_store_btc_on_eth_contract_address,
            get_eth_core_init_output_json::get_btc_on_eth_eth_core_init_output_json,
            initialize_eth_core::initialize_eth_core,
        },
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
    check_for_existence_of_eth_contract_byte_code(bytecode_path)
        .map(|_| EthState::init(db))
        .and_then(|state| match is_eth_core_initialized(&state.db) {
            true => Ok("{eth_core_initialized:true}".to_string()),
            false => initialize_eth_core(
                block_json,
                chain_id,
                gas_price,
                canon_to_tip_length,
                bytecode_path,
                state,
            )
            .and_then(generate_and_store_btc_on_eth_contract_address)
            .and_then(get_btc_on_eth_eth_core_init_output_json),
        })
}
