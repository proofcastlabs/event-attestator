use common::{
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use common_eth::{
    end_eth_db_transaction_and_return_state,
    parse_eth_submission_material_and_put_in_state,
    validate_eth_block_in_state,
    validate_receipts_in_state,
    EthDbUtilsExt,
    EthState,
};
use function_name::named;

use crate::{
    constants::CORE_TYPE,
    eth::{
        account_for_fees_in_eth_tx_infos_in_state,
        filter_receipts_for_eos_on_eth_eth_tx_info_in_state,
        get_output_json,
        maybe_divert_eos_txs_to_safe_address_if_destination_is_token_address,
        maybe_filter_out_eth_tx_info_with_value_too_low_in_state,
        maybe_filter_out_zero_eos_asset_amounts_in_state,
        maybe_increment_eos_account_nonce_and_return_state,
        maybe_sign_eos_txs_and_add_to_eth_state,
        update_accrued_fees_in_dictionary_and_return_eth_state,
        EosOnEthEosTxInfos,
    },
};

#[named]
fn reprocess_eth_block<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    accrue_fees: bool,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug reprocessing ETH block...");
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json, &accrue_fees)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| parse_eth_submission_material_and_put_in_state(block_json, EthState::init(db)))
        .and_then(validate_eth_block_in_state)
        .and_then(|state| state.get_eos_eth_token_dictionary_from_db_and_add_to_state())
        .and_then(validate_receipts_in_state)
        .and_then(filter_receipts_for_eos_on_eth_eth_tx_info_in_state)
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
                EosOnEthEosTxInfos::from_eth_submission_material(
                    state.get_eth_submission_material()?,
                    state.get_eos_eth_token_dictionary()?,
                    &state.eth_db_utils.get_eth_chain_id_from_db()?,
                )
                .and_then(|tx_infos| tx_infos.to_bytes())
                .map(|bytes| state.add_tx_infos(bytes))
            }
        })
        .and_then(maybe_filter_out_eth_tx_info_with_value_too_low_in_state)
        .and_then(account_for_fees_in_eth_tx_infos_in_state)
        .and_then(maybe_filter_out_zero_eos_asset_amounts_in_state)
        .and_then(|state| {
            if accrue_fees {
                update_accrued_fees_in_dictionary_and_return_eth_state(state)
            } else {
                info!("✘ Not accruing fees during ETH block reprocessing...");
                Ok(state)
            }
        })
        .and_then(maybe_divert_eos_txs_to_safe_address_if_destination_is_token_address)
        .and_then(maybe_sign_eos_txs_and_add_to_eth_state)
        .and_then(maybe_increment_eos_account_nonce_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_output_json)
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Reprocess ETH Block For Stale EOS Transaction
///
/// This function will take a passed in ETH block submission material and run it through the
/// simplified submission pipeline, signing any EOS signatures for peg-ins it may find in the block
///
/// ### NOTES:
///  - This version of the ETH block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, but it will __not__ accrue those fees on to the total in the
///  dictionary. This is to avoid accounting for fees twice.
///
/// ### BEWARE:
/// This function WILL increment the EOS nonce if transactions are signed. The user of this function
/// should understand what this means when inserting the report outputted from this debug function.
/// If this output is to _replace_ an existing report, the nonces in the report and in the core's
/// database should be modified accordingly.
pub fn debug_reprocess_eth_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    reprocess_eth_block(db, block_json, false, signature)
}

/// # Debug Reprocess ETH Block For Stale EOS Transaction With Fee Accrual
///
/// This function will take a passed in ETH block submission material and run it through the
/// simplified submission pipeline, signing any EOS signatures for peg-ins it may find in the block
///
/// ### NOTES:
///  - This version of the ETH block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, and __will__ accrue those fees on to the total in the
///  dictionary. Only use this is you know what you're doing and why, and make sure you're avoiding
///  accruing the fees twice if the block has already been processed through the non-debug EVM
///  block submission pipeline.
///
/// ### BEWARE:
/// This function WILL increment the EOS nonce if transactions are signed. The user of this function
/// should understand what this means when inserting the report outputted from this debug function.
/// If this output is to _replace_ an existing report, the nonces in the report and in the core's
/// database should be modified accordingly.
pub fn debug_reprocess_eth_block_with_fee_accrual<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    signature: &str,
) -> Result<String> {
    reprocess_eth_block(db, block_json, true, signature)
}
