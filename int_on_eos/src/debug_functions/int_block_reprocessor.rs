use common::{
    chains::eth::{
        eth_database_transactions::end_eth_db_transaction_and_return_state,
        eth_database_utils::{EthDbUtils, EthDbUtilsExt},
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        increment_eos_account_nonce::maybe_increment_eos_account_nonce_and_return_state,
        validate_block_in_state::validate_eth_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
    },
    core_type::CoreType,
    debug_functions::validate_debug_command_signature,
    dictionaries::eos_eth::{get_eos_eth_token_dictionary_from_db_and_add_to_eth_state, EosEthTokenDictionary},
    state::EthState,
    traits::DatabaseInterface,
    types::Result,
};
use function_name::named;
pub use serde_json::json;

use crate::{
    constants::CORE_TYPE,
    int::{
        debug_filter_tx_info_with_no_erc20_transfer_event,
        filter_out_zero_value_eos_tx_infos_from_state,
        filter_submission_material_for_relevant_receipts_in_state,
        get_output_json,
        maybe_sign_eos_txs_and_add_to_eth_state,
        IntOnEosEosTxInfos,
    },
};

#[named]
fn reprocess_int_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    info!("✔ Debug reprocessing INT block...");
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| parse_eth_submission_material_and_put_in_state(block_json, EthState::init(db)))
        .and_then(CoreType::check_core_is_initialized_and_return_eth_state)
        .and_then(validate_eth_block_in_state)
        .and_then(get_eos_eth_token_dictionary_from_db_and_add_to_eth_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_relevant_receipts_in_state)
        .and_then(|state| {
            let submission_material = state.get_eth_submission_material()?.clone();
            if submission_material.receipts.is_empty() {
                info!("✔ No receipts in block ∴ no info to parse!");
                Ok(state)
            } else {
                info!(
                    "✔ {} receipts in block ∴ parsing info...",
                    submission_material.get_block_number()?
                );
                EosEthTokenDictionary::get_from_db(state.db)
                    .and_then(|token_dictionary| {
                        let int_db_utils = &EthDbUtils::new(db);
                        IntOnEosEosTxInfos::from_submission_material(
                            &submission_material,
                            &token_dictionary,
                            &int_db_utils.get_eth_chain_id_from_db()?,
                            &int_db_utils.get_eth_router_smart_contract_address_from_db()?,
                            &int_db_utils.get_int_on_eos_smart_contract_address_from_db()?,
                        )
                    })
                    .and_then(|tx_infos| tx_infos.to_bytes())
                    .map(|bytes| state.add_tx_infos(bytes))
            }
        })
        .and_then(filter_out_zero_value_eos_tx_infos_from_state)
        .and_then(debug_filter_tx_info_with_no_erc20_transfer_event)
        .and_then(maybe_sign_eos_txs_and_add_to_eth_state)
        .and_then(maybe_increment_eos_account_nonce_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_output_json)
        .map(|output| output.to_string())
}

/// # Debug Reprocess INT Block For Stale EOS Transaction
///
/// This function will take a passed in INT block submission material and run it through the
/// simplified submission pipeline, signing any EOS signatures for peg-ins it may find in the block
///
/// ### BEWARE:
/// This function WILL increment the EOS nonce if transactions are signed. The user of this function
/// should understand what this means when inserting the report outputted from this debug function.
/// If this output is to _replace_ an existing report, the nonces in the report and in the core's
/// database should be modified accordingly.
pub fn debug_reprocess_int_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    reprocess_int_block(db, block_json, signature)
}
