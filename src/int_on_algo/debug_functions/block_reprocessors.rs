use crate::{
    chains::{
        algo::{
            algo_database_transactions::{
                end_algo_db_transaction_and_return_state,
                start_algo_db_transaction_and_return_state,
            },
            algo_state::AlgoState,
            algo_submission_material::parse_algo_submission_material_and_put_in_state,
            increment_eth_account_nonce::maybe_increment_eth_account_nonce_and_return_algo_state,
            remove_all_txs_from_submission_material_in_state::remove_all_txs_from_submission_material_in_state,
        },
        eth::eth_database_utils::EthDbUtilsExt,
    },
    dictionaries::evm_algo::get_evm_algo_token_dictionary_and_add_to_algo_state,
    int_on_algo::{
        algo::{
            add_relevant_txs_to_submission_material::add_relevant_validated_txs_to_submission_material_in_state,
            filter_zero_value_tx_infos::filter_out_zero_value_tx_infos_from_state,
            get_algo_output::get_algo_output,
            get_relevant_txs::get_relevant_asset_txs_from_submission_material_and_add_to_state,
            int_tx_info::IntOnAlgoIntTxInfos,
            parse_tx_info::maybe_parse_tx_info_from_canon_block_and_add_to_state,
            sign_txs::maybe_sign_int_txs_and_add_to_algo_state,
            validate_relevant_txs::filter_out_invalid_txs_and_update_in_state,
        },
        check_core_is_initialized::check_core_is_initialized_and_return_algo_state,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn debug_reprocess_algo_block<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Debug reprocessing ALGO block...");
    parse_algo_submission_material_and_put_in_state(block_json_string, AlgoState::init(db))
        .and_then(check_core_is_initialized_and_return_algo_state)
        .and_then(start_algo_db_transaction_and_return_state)
        .and_then(get_evm_algo_token_dictionary_and_add_to_algo_state)
        .and_then(get_relevant_asset_txs_from_submission_material_and_add_to_state)
        .and_then(filter_out_invalid_txs_and_update_in_state)
        .and_then(remove_all_txs_from_submission_material_in_state)
        .and_then(add_relevant_validated_txs_to_submission_material_in_state)
        .and_then(|state| {
            let material = state.get_algo_submission_material()?;
            match material.block.transactions {
                None => {
                    info!("✔ No transactions in canon submission material ∴ no tx info to parse!");
                    Ok(state)
                },
                Some(txs) => {
                    info!(
                        "✔ {} transactions in canon submission material ∴ parsing info...",
                        txs.len()
                    );
                    let dictionary = state.get_evm_algo_token_dictionary()?;
                    let tx_infos = IntOnAlgoIntTxInfos::from_algo_txs(
                        &txs,
                        &dictionary,
                        &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                        &state.algo_db_utils.get_genesis_hash()?,
                        &state.eth_db_utils.get_int_on_algo_smart_contract_address()?,
                    )?;
                    state.add_int_on_algo_int_tx_infos(tx_infos)
                },
            }
        })
        .and_then(filter_out_zero_value_tx_infos_from_state)
        //.and_then(maybe_divert_txs_to_safe_address_if_destination_is_evm_token_address) // TODO this!
        .and_then(maybe_sign_int_txs_and_add_to_algo_state)
        .and_then(maybe_increment_eth_account_nonce_and_return_algo_state)
        .and_then(end_algo_db_transaction_and_return_state)
        .and_then(get_algo_output)
}
