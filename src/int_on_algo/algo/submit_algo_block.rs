#![allow(unused_imports)] // FIXM rm!

use crate::{
    chains::algo::{
        add_latest_algo_block::add_latest_algo_block_to_db_and_return_state,
        algo_database_transactions::{
            end_algo_db_transaction_and_return_state,
            start_algo_db_transaction_and_return_state,
        },
        algo_state::AlgoState,
        algo_submission_material::parse_algo_submission_material_and_put_in_state,
        check_parent_exists::check_parent_of_algo_block_in_state_exists,
        increment_eth_account_nonce::maybe_increment_eth_account_nonce_and_return_algo_state,
        remove_irrelevant_txs_from_block_in_state::remove_irrelevant_txs_from_block_in_state,
        remove_old_algo_tail_block::maybe_remove_old_algo_tail_block_and_return_state,
        remove_receipts_from_canon_block::maybe_remove_receipts_from_algo_canon_block_and_return_state,
        update_algo_canon_block_hash::maybe_update_algo_canon_block_hash_and_return_state,
        update_algo_linker_hash::maybe_update_algo_linker_hash_and_return_state,
        update_algo_tail_block_hash::maybe_update_algo_tail_block_hash_and_return_state,
    },
    dictionaries::evm_algo::get_evm_algo_token_dictionary_and_add_to_algo_state,
    int_on_algo::{
        algo::{
            filter_zero_value_tx_infos::filter_out_zero_value_tx_infos_from_state,
            get_algo_output::get_algo_output,
            parse_tx_info::maybe_parse_tx_info_from_canon_block_and_add_to_state,
            sign_txs::maybe_sign_int_txs_and_add_to_algo_state,
        },
        check_core_is_initialized::check_core_is_initialized_and_return_algo_state,
    },
    traits::DatabaseInterface,
    types::Result,
};

/// Submit Algo Block To Core
///
/// The main submission pipeline. Submitting an Algorand block to the enclave will - if that block is
/// valid & subsequent to the enclave's current latest block - advanced the piece of the ALGO
/// blockchain held by the enclave in it's encrypted database. Should the submitted block
/// contain pertinent transactions to the redeem addres  the enclave is watching, an INT
/// transaction will be signed & returned to the caller.
pub fn submit_algo_block_to_core<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    info!("✔ Submitting ALGO block to core...");
    parse_algo_submission_material_and_put_in_state(block_json_string, AlgoState::init(&db))
        .and_then(check_core_is_initialized_and_return_algo_state)
        .and_then(start_algo_db_transaction_and_return_state)
        .and_then(get_evm_algo_token_dictionary_and_add_to_algo_state)
        .and_then(check_parent_of_algo_block_in_state_exists)
        //.and_then(validate_transactions_in_state) // FIXME Only do this is there's one we care about?
        .and_then(remove_irrelevant_txs_from_block_in_state)
        .and_then(add_latest_algo_block_to_db_and_return_state)
        .and_then(maybe_update_algo_canon_block_hash_and_return_state)
        .and_then(maybe_update_algo_tail_block_hash_and_return_state)
        .and_then(maybe_update_algo_linker_hash_and_return_state)
        .and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        .and_then(filter_out_zero_value_tx_infos_from_state)
        //.and_then(maybe_divert_txs_to_safe_address_if_destinajtion_is_evm_token_address) // TODO this!
        .and_then(maybe_sign_int_txs_and_add_to_algo_state)
        .and_then(maybe_increment_eth_account_nonce_and_return_algo_state)
        .and_then(maybe_remove_old_algo_tail_block_and_return_state)
        .and_then(maybe_remove_receipts_from_algo_canon_block_and_return_state)
        .and_then(end_algo_db_transaction_and_return_state)
        .and_then(get_algo_output) // FIXME Implement this fully!
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::{
            algo::{
                core_initialization::initialize_algo_core::initialize_algo_core,
                test_utils::get_sample_contiguous_blocks,
            },
            eth::{
                core_initialization::initialize_eth_core::initialize_eth_core_with_no_contract_tx,
                eth_chain_id::EthChainId,
                eth_state::EthState,
                eth_test_utils::get_sample_eth_submission_material_string,
            },
        },
        dictionaries::evm_algo::EvmAlgoTokenDictionary,
        test_utils::get_test_database,
    };

    #[test]
    fn should_submit_algo_block_successfully() {
        let db = get_test_database();
        let canon_to_tip_length = 3;
        let algo_fee = 1000;
        let genesis_id = "mainnet-v1.0";
        let algo_block_json_strings = get_sample_contiguous_blocks()
            .iter()
            .map(|block| block.to_string())
            .collect::<Vec<String>>();
        let state = AlgoState::init_with_empty_dictionary(&db);
        initialize_algo_core(
            state,
            &algo_block_json_strings[0],
            algo_fee,
            canon_to_tip_length,
            genesis_id,
        )
        .unwrap();
        let eth_block_json_string = get_sample_eth_submission_material_string(0).unwrap();
        let eth_chain_id = EthChainId::Ropsten;
        let eth_gas_price = 20_000_000_000;
        initialize_eth_core_with_no_contract_tx(
            &eth_block_json_string,
            &eth_chain_id,
            eth_gas_price,
            canon_to_tip_length,
            EthState::init(&db),
        )
        .unwrap();
        submit_algo_block_to_core(db, &algo_block_json_strings[1]).unwrap();
    }
}
