use crate::{
    chains::eth::{
        core_initialization::{
            eth_core_init_utils::{
                add_eth_block_to_db_and_return_state,
                add_evm_block_to_db_and_return_state,
                put_eth_account_nonce_in_db_and_return_state,
                put_eth_any_sender_nonce_in_db_and_return_state,
                put_eth_canon_to_tip_length_in_db_and_return_state,
                put_eth_chain_id_in_db_and_return_state,
                put_eth_gas_price_in_db_and_return_state,
                put_eth_tail_block_hash_in_db_and_return_state,
                put_evm_account_nonce_in_db_and_return_state,
                put_evm_any_sender_nonce_in_db_and_return_state,
                put_evm_canon_to_tip_length_in_db_and_return_state,
                put_evm_chain_id_in_db_and_return_state,
                put_evm_gas_price_in_db_and_return_state,
                put_evm_tail_block_hash_in_db_and_return_state,
                remove_receipts_from_block_in_state,
                set_eth_anchor_block_hash_and_return_state,
                set_eth_canon_block_hash_and_return_state,
                set_eth_latest_block_hash_and_return_state,
                set_evm_anchor_block_hash_and_return_state,
                set_evm_canon_block_hash_and_return_state,
                set_evm_latest_block_hash_and_return_state,
            },
            generate_eth_address::{generate_and_store_eth_address, generate_and_store_evm_address},
            generate_eth_contract_tx::{
                generate_eth_contract_tx_and_put_in_state,
                generate_evm_contract_tx_and_put_in_state,
            },
            generate_eth_private_key::{generate_and_store_eth_private_key, generate_and_store_evm_private_key},
        },
        eth_chain_id::EthChainId,
        eth_state::EthState,
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        validate_block_in_state::validate_block_in_state,
    },
    traits::DatabaseInterface,
    types::Result,
};

fn initialize_eth_core_maybe_with_contract_tx<'a, D: DatabaseInterface>(
    block_json: &str,
    chain_id: &EthChainId,
    gas_price: u64,
    canon_to_tip_length: u64,
    maybe_bytecode_path: Option<&str>,
    state: EthState<'a, D>,
    is_for_eth: bool,
) -> Result<EthState<'a, D>> {
    parse_eth_submission_material_and_put_in_state(block_json, state)
        .and_then(|state| {
            if is_for_eth {
                put_eth_chain_id_in_db_and_return_state(chain_id, state)
            } else {
                put_evm_chain_id_in_db_and_return_state(chain_id, state)
            }
        })
        .and_then(validate_block_in_state)
        .and_then(remove_receipts_from_block_in_state)
        .and_then(|state| {
            if is_for_eth {
                add_eth_block_to_db_and_return_state(state)
            } else {
                add_evm_block_to_db_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_canon_to_tip_length_in_db_and_return_state(canon_to_tip_length, state)
            } else {
                put_evm_canon_to_tip_length_in_db_and_return_state(canon_to_tip_length, state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                set_eth_anchor_block_hash_and_return_state(state)
            } else {
                set_evm_anchor_block_hash_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                set_eth_latest_block_hash_and_return_state(state)
            } else {
                set_evm_latest_block_hash_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                set_eth_canon_block_hash_and_return_state(state)
            } else {
                set_evm_canon_block_hash_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                generate_and_store_eth_private_key(state)
            } else {
                generate_and_store_evm_private_key(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_tail_block_hash_in_db_and_return_state(state)
            } else {
                put_evm_tail_block_hash_in_db_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_gas_price_in_db_and_return_state(gas_price, state)
            } else {
                put_evm_gas_price_in_db_and_return_state(gas_price, state)
            }
        })
        .and_then(|state| match maybe_bytecode_path {
            Some(_) => {
                if is_for_eth {
                    put_eth_account_nonce_in_db_and_return_state(state, 1)
                } else {
                    put_evm_account_nonce_in_db_and_return_state(state, 1)
                }
            },
            None => {
                if is_for_eth {
                    put_eth_account_nonce_in_db_and_return_state(state, 0)
                } else {
                    put_evm_account_nonce_in_db_and_return_state(state, 0)
                }
            },
        })
        .and_then(|state| {
            if is_for_eth {
                put_eth_any_sender_nonce_in_db_and_return_state(state)
            } else {
                put_evm_any_sender_nonce_in_db_and_return_state(state)
            }
        })
        .and_then(|state| {
            if is_for_eth {
                generate_and_store_eth_address(state)
            } else {
                generate_and_store_evm_address(state)
            }
        })
        .and_then(|state| match maybe_bytecode_path {
            Some(path) => {
                if is_for_eth {
                    generate_eth_contract_tx_and_put_in_state(chain_id, gas_price, path, state)
                } else {
                    generate_evm_contract_tx_and_put_in_state(chain_id, gas_price, path, state)
                }
            },
            None => Ok(state),
        })
}

pub fn initialize_eth_core_with_no_contract_tx<'a, D: DatabaseInterface>(
    block_json: &str,
    chain_id: &EthChainId,
    gas_price: u64,
    canon_to_tip_length: u64,
    state: EthState<'a, D>,
) -> Result<EthState<'a, D>> {
    info!("✔ Initializing ETH core with NO contract tx...");
    initialize_eth_core_maybe_with_contract_tx(block_json, chain_id, gas_price, canon_to_tip_length, None, state, true)
}

pub fn initialize_evm_core_with_no_contract_tx<'a, D: DatabaseInterface>(
    block_json: &str,
    chain_id: &EthChainId,
    gas_price: u64,
    canon_to_tip_length: u64,
    state: EthState<'a, D>,
) -> Result<EthState<'a, D>> {
    info!("✔ Initializing EVM core with NO contract tx...");
    initialize_eth_core_maybe_with_contract_tx(block_json, chain_id, gas_price, canon_to_tip_length, None, state, false)
}
