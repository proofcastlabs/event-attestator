use crate::{
    chains::eth::{
        eth_database_transactions::{
            end_eth_db_transaction_and_return_state,
            start_eth_db_transaction_and_return_state,
        },
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        increment_algo_account_nonce::maybe_increment_algo_account_nonce_and_return_eth_state,
        validate_block_in_state::validate_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
    },
    dictionaries::evm_algo::get_evm_algo_token_dictionary_and_add_to_eth_state,
    int_on_algo::{
        check_core_is_initialized::check_core_is_initialized_and_return_eth_state,
        int::{
            algo_tx_info::IntOnAlgoAlgoTxInfos,
            filter_submission_material::filter_submission_material_for_peg_in_events_in_state,
            filter_zero_value_tx_infos::filter_out_zero_value_tx_infos_from_state,
            get_int_output_json::get_int_output_json,
            sign_txs::maybe_sign_algo_txs_and_add_to_state,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

/// # Debug Reprocess INT Block
///
/// This function will take a passed in INT block submission material and run it through the
/// submission pipeline, signing any signatures for peg-ins it may find in the block
///
/// ### NOTES:
///
///  - This function will increment the core's ALGO nonce by the number of txs signed.
/// gap in their report IDs!
pub fn debug_reprocess_int_block<D: DatabaseInterface>(db: &D, block_json_string: &str) -> Result<String> {
    info!("✔ Debug reprocessing INT block...");
    parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(db))
        .and_then(check_core_is_initialized_and_return_eth_state)
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_block_in_state)
        .and_then(get_evm_algo_token_dictionary_and_add_to_eth_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(|state| {
            let submission_material = state.get_eth_submission_material()?;
            match submission_material.receipts.is_empty() {
                true => {
                    info!("✔ No receipts in canon block ∴ no info to parse!");
                    Ok(state)
                },
                false => {
                    info!(
                        "✔ {} receipts in canon block ∴ parsing info...",
                        submission_material.receipts.len()
                    );
                    let tx_infos = IntOnAlgoAlgoTxInfos::from_submission_material(
                        submission_material,
                        &state.eth_db_utils.get_int_on_algo_smart_contract_address()?,
                        state.get_evm_algo_token_dictionary()?,
                        &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                    )?;
                    state.add_int_on_algo_algo_tx_infos(tx_infos)
                },
            }
        })
        .and_then(filter_out_zero_value_tx_infos_from_state)
        .and_then(maybe_sign_algo_txs_and_add_to_state)
        .and_then(maybe_increment_algo_account_nonce_and_return_eth_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_int_output_json)
}
