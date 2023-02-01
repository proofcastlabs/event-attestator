use function_name::named;

use crate::{
    chains::eth::{
        eth_database_transactions::end_eth_db_transaction_and_return_state,
        eth_database_utils::EthDbUtilsExt,
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        increment_algo_account_nonce::maybe_increment_algo_account_nonce_and_return_eth_state,
        validate_block_in_state::validate_eth_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
    },
    core_type::CoreType,
    debug_functions::validate_debug_command_signature,
    dictionaries::evm_algo::get_evm_algo_token_dictionary_and_add_to_eth_state,
    int_on_algo::{
        constants::CORE_TYPE,
        int::{
            debug_filter_tx_info_with_no_erc20_transfer_event,
            filter_out_zero_value_tx_infos_from_state,
            filter_submission_material_for_peg_in_events_in_state,
            get_int_output_json,
            maybe_sign_algo_txs_and_add_to_state,
            IntOnAlgoAlgoTxInfos,
        },
    },
    state::EthState,
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
#[named]
pub fn debug_reprocess_int_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    info!("✔ Debug reprocessing INT block...");
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| parse_eth_submission_material_and_put_in_state(block_json, EthState::init(db)))
        .and_then(CoreType::check_core_is_initialized_and_return_eth_state)
        .and_then(validate_eth_block_in_state)
        .and_then(get_evm_algo_token_dictionary_and_add_to_eth_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(|state| {
            let submission_material = state.get_eth_submission_material()?;
            if submission_material.receipts.is_empty() {
                info!("✔ No receipts in canon block ∴ no info to parse!");
                Ok(state)
            } else {
                info!(
                    "✔ {} receipts in canon block ∴ parsing info...",
                    submission_material.receipts.len()
                );
                let tx_infos = IntOnAlgoAlgoTxInfos::from_submission_material(
                    submission_material,
                    &state.eth_db_utils.get_int_on_algo_smart_contract_address()?,
                    state.get_evm_algo_token_dictionary()?,
                    &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                    &state.algo_db_utils.get_algo_app_id()?,
                )?;
                state.add_int_on_algo_algo_tx_infos(tx_infos)
            }
        })
        .and_then(filter_out_zero_value_tx_infos_from_state)
        .and_then(debug_filter_tx_info_with_no_erc20_transfer_event)
        .and_then(maybe_sign_algo_txs_and_add_to_state)
        .and_then(maybe_increment_algo_account_nonce_and_return_eth_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_int_output_json)
        .map(|output| output.to_string())
}
