pub use serde_json::json;

use crate::{
    chains::eth::{
        eth_database_transactions::{
            end_eth_db_transaction_and_return_state,
            start_eth_db_transaction_and_return_state,
        },
        eth_database_utils::EthDbUtilsExt,
        eth_state::EthState,
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        increment_eos_account_nonce::maybe_increment_eos_account_nonce_and_return_state,
        validate_block_in_state::validate_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
    },
    check_debug_mode::check_debug_mode,
    dictionaries::eos_eth::get_eos_eth_token_dictionary_from_db_and_add_to_eth_state,
    eos_on_int::{
        check_core_is_initialized::check_core_is_initialized_and_return_eth_state,
        int::{
            divert_to_safe_address::maybe_divert_txs_to_safe_address_if_destination_is_token_address,
            eos_tx_info::EosOnIntEosTxInfos,
            filter_receipts_in_state::filter_receipts_for_eos_on_int_eos_tx_info_in_state,
            filter_tx_info::{
                maybe_filter_out_int_tx_info_with_value_too_low_in_state,
                maybe_filter_out_zero_eos_asset_amounts_in_state,
            },
            get_int_output::get_int_output,
            sign_txs::maybe_sign_eos_txs_and_add_to_eth_state,
        },
    },
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

fn reprocess_int_block<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    info!("✔ Debug reprocessing INT block...");
    check_debug_mode()
        .and_then(|_| parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(&db)))
        .and_then(check_core_is_initialized_and_return_eth_state)
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_block_in_state)
        .and_then(get_eos_eth_token_dictionary_from_db_and_add_to_eth_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_receipts_for_eos_on_int_eos_tx_info_in_state)
        .and_then(|state| {
            let submission_material = state.get_eth_submission_material()?.clone();
            if submission_material.receipts.is_empty() {
                info!("✔ No receipts in block ∴ no info to parse!");
                Ok(state)
            } else {
                info!(
                    "✔ {} receipts in block ∴ parsing info...",
                    submission_material.get_num_receipts()
                );
                EosOnIntEosTxInfos::from_int_submission_material(
                    state.get_eth_submission_material()?,
                    state.get_eos_eth_token_dictionary()?,
                    &state.eth_db_utils.get_eth_chain_id_from_db()?,
                    &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                )
                .and_then(|tx_infos| state.add_eos_on_int_eos_tx_infos(tx_infos))
            }
        })
        .and_then(maybe_filter_out_int_tx_info_with_value_too_low_in_state)
        .and_then(maybe_filter_out_zero_eos_asset_amounts_in_state)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(maybe_sign_eos_txs_and_add_to_eth_state)
        .and_then(maybe_increment_eos_account_nonce_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_int_output)
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Reprocess INT Block For Stale EOS Transaction
///
/// This function will take a passed in INT block submission material and run it through the
/// simplified submission pipeline, signing any EOS signatures for peg-ins it may find in the block
///
/// ### BEWARE:
///
/// This function WILL increment the EOS nonce if transactions are signed. The user of this function
/// should understand what this means when inserting the report outputted from this debug function.
/// If this output is to _replace_ an existing report, the nonces in the report and in the core's
/// database should be modified accordingly.
pub fn debug_reprocess_int_block<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    reprocess_int_block(db, block_json_string)
}