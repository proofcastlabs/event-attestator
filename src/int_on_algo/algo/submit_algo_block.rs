#![allow(unused_imports)] // FIXM rm!

use crate::{
    chains::algo::{
        add_latest_algo_block::add_latest_algo_block_and_return_state,
        algo_database_transactions::{
            end_algo_db_transaction_and_return_state,
            start_algo_db_transaction_and_return_state,
        },
        algo_state::AlgoState,
        algo_submission_material::parse_algo_submission_material_and_put_in_state,
        check_parent_exists::check_parent_of_algo_block_in_state_exists,
        remove_old_algo_tail_block::maybe_remove_old_algo_tail_block_and_return_state,
        update_algo_canon_block_hash::maybe_update_algo_canon_block_hash_and_return_state,
        update_algo_linker_hash::maybe_update_algo_linker_hash_and_return_state,
        update_algo_tail_block_hash::maybe_update_algo_tail_block_hash_and_return_state,
    },
    dictionaries::evm_algo::get_evm_algo_token_dictionary_and_add_to_algo_state,
    int_on_algo::{
        algo::get_algo_output::get_algo_output,
        check_core_is_initialized::check_core_is_initialized_and_return_algo_state,
    },
    traits::DatabaseInterface,
    types::Result,
};
// So the setup will be thus:
// Anyone can create an asset, with this enclave address as the:
// Manager account - the only account how can change/reconfigure the asset
// Reserve address - where the created tokens go to (instead of the creator account). Transfers out of here are "mints", and transfers back to here are redeems.
// Freeze address - either this enclave or empty string. This account can then freeze people.
// Clawback address - either this enclave or empty string.
//
// Setting the reserve address to this enclave can ONLY be done if this enclave has signed a tx
// saying it's happy to accept the asset. Nice.
//
// So then the dictionary will be a list of INT vault tokens mapped to asset IDs. Now the enclave
// can search for transactions where it is the recipient of some asset, which will count as a
// redeem and proceed from there.
//
// The other side of the enclave can then use this address to sign txs to send the asset from this
// reserver account, which counts as a mint.
//
// Nice!

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
        //.and_then(validate_block_in_state) // FIXME Maybe validate receipts
        .and_then(get_evm_algo_token_dictionary_and_add_to_algo_state)
        .and_then(check_parent_of_algo_block_in_state_exists)
        //.and_then(validate_receipts_in_state)
        //.and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(add_latest_algo_block_and_return_state)
        .and_then(maybe_update_algo_canon_block_hash_and_return_state)
        .and_then(maybe_update_algo_tail_block_hash_and_return_state)
        .and_then(maybe_update_algo_linker_hash_and_return_state)
        //.and_then(maybe_parse_tx_info_from_canon_block_and_add_to_state)
        //.and_then(filter_out_zero_value_evm_tx_infos_from_state)
        //.and_then(maybe_account_for_fees)
        //.and_then(maybe_divert_txs_to_safe_address_if_destination_is_evm_token_address)
        //.and_then(maybe_sign_evm_txs_and_add_to_eth_state)
        //.and_then(maybe_increment_evm_account_nonce_and_return_eth_state)
        .and_then(maybe_remove_old_algo_tail_block_and_return_state)
        //.and_then(maybe_remove_receipts_from_eth_canon_block_and_return_state)
        .and_then(end_algo_db_transaction_and_return_state)
        .and_then(get_algo_output)
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
        let dict = EvmAlgoTokenDictionary::default();
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
