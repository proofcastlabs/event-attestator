pub use serde_json::json;

use crate::{
    chains::{
        eos::{
            add_schedule::maybe_add_new_eos_schedule_to_db_and_return_state,
            eos_constants::REDEEM_ACTION_NAME,
            eos_database_transactions::{
                end_eos_db_transaction_and_return_state,
                start_eos_db_transaction_and_return_state,
            },
            eos_database_utils::get_latest_eos_block_number,
            eos_global_sequences::{
                get_processed_global_sequences_and_add_to_state,
                maybe_add_global_sequences_to_processed_list_and_return_state,
            },
            eos_state::EosState,
            eos_submission_material::parse_submission_material_and_add_to_state,
            filter_action_proofs::{
                maybe_filter_duplicate_proofs_from_state,
                maybe_filter_out_action_proof_receipt_mismatches_and_return_state,
                maybe_filter_out_invalid_action_receipt_digests,
                maybe_filter_out_proofs_for_accounts_not_in_token_dictionary,
                maybe_filter_out_proofs_with_invalid_merkle_proofs,
                maybe_filter_out_proofs_with_wrong_action_mroot,
                maybe_filter_proofs_for_action_name,
            },
            get_enabled_protocol_features::get_enabled_protocol_features_and_add_to_state,
        },
        eth::{
            eth_database_transactions::{
                end_eth_db_transaction_and_return_state,
                start_eth_db_transaction_and_return_state,
            },
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
            eth_state::EthState,
            eth_submission_material::parse_eth_submission_material_and_put_in_state,
            increment_eos_account_nonce::maybe_increment_eos_account_nonce_and_return_state,
            validate_block_in_state::validate_block_in_state,
            validate_receipts_in_state::validate_receipts_in_state,
        },
    },
    dictionaries::eos_eth::{
        get_eos_eth_token_dictionary_from_db_and_add_to_eos_state,
        get_eos_eth_token_dictionary_from_db_and_add_to_eth_state,
        EosEthTokenDictionary,
    },
    erc20_on_eos::{
        check_core_is_initialized::{
            check_core_is_initialized_and_return_eos_state,
            check_core_is_initialized_and_return_eth_state,
        },
        eos::{
            account_for_fees::{
                account_for_fees_in_redeem_infos_in_state,
                update_accrued_fees_in_dictionary_and_return_eos_state,
            },
            get_eos_output::{get_eth_signed_tx_info_from_eth_txs, EosOutput},
            increment_eth_nonce::maybe_increment_eth_nonce_in_db_and_return_eos_state,
            redeem_info::maybe_parse_redeem_infos_and_put_in_state,
            sign_normal_eth_txs::get_eth_signed_txs,
        },
        eth::{
            account_for_fees::{
                account_for_fees_in_peg_in_infos_in_state,
                update_accrued_fees_in_dictionary_and_return_eth_state,
            },
            get_output_json::get_output_json,
            peg_in_info::{
                filter_out_zero_value_peg_ins_from_state,
                filter_submission_material_for_peg_in_events_in_state,
                maybe_sign_eos_txs_and_add_to_eth_state,
                Erc20OnEosPegInInfos,
            },
        },
    },
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

fn reprocess_eth_block<D: DatabaseInterface>(db: D, block_json_string: &str, accrue_fees: bool) -> Result<String> {
    info!("✔ Debug reprocessing ETH block...");
    parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(&db))
        .and_then(check_core_is_initialized_and_return_eth_state)
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_block_in_state)
        .and_then(get_eos_eth_token_dictionary_from_db_and_add_to_eth_state)
        .and_then(validate_receipts_in_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(|state| {
            let submission_material = state.get_eth_submission_material()?.clone();
            match submission_material.receipts.is_empty() {
                true => {
                    info!("✔ No receipts in block ∴ no info to parse!");
                    Ok(state)
                },
                false => {
                    info!(
                        "✔ {} receipts in block ∴ parsing info...",
                        submission_material.get_block_number()?
                    );
                    EosEthTokenDictionary::get_from_db(state.db)
                        .and_then(|token_dictionary| {
                            Erc20OnEosPegInInfos::from_submission_material(
                                &submission_material,
                                &token_dictionary,
                                &EthDbUtils::new(&db).get_eth_chain_id_from_db()?,
                            )
                        })
                        .and_then(|peg_in_infos| state.add_erc20_on_eos_peg_in_infos(peg_in_infos))
                        .and_then(filter_out_zero_value_peg_ins_from_state)
                },
            }
        })
        .and_then(account_for_fees_in_peg_in_infos_in_state)
        .and_then(filter_out_zero_value_peg_ins_from_state)
        .and_then(|state| {
            if accrue_fees {
                update_accrued_fees_in_dictionary_and_return_eth_state(state)
            } else {
                info!("✘ Not accruing fees during ETH block reprocessing...");
                Ok(state)
            }
        })
        .and_then(maybe_sign_eos_txs_and_add_to_eth_state)
        .and_then(maybe_increment_eos_account_nonce_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_output_json)
}

fn reprocess_eos_block<D: DatabaseInterface>(
    db: D,
    block_json: &str,
    accrue_fees: bool,
    maybe_nonce: Option<u64>,
) -> Result<String> {
    info!("✔ Debug reprocessing EOS block...");
    parse_submission_material_and_add_to_state(block_json, EosState::init(&db))
        .and_then(check_core_is_initialized_and_return_eos_state)
        .and_then(get_enabled_protocol_features_and_add_to_state)
        .and_then(start_eos_db_transaction_and_return_state)
        .and_then(get_processed_global_sequences_and_add_to_state)
        .and_then(get_eos_eth_token_dictionary_from_db_and_add_to_eos_state)
        .and_then(maybe_add_new_eos_schedule_to_db_and_return_state)
        .and_then(maybe_filter_duplicate_proofs_from_state)
        .and_then(maybe_filter_out_proofs_for_accounts_not_in_token_dictionary)
        .and_then(maybe_filter_out_action_proof_receipt_mismatches_and_return_state)
        .and_then(maybe_filter_out_invalid_action_receipt_digests)
        .and_then(maybe_filter_out_proofs_with_invalid_merkle_proofs)
        .and_then(maybe_filter_out_proofs_with_wrong_action_mroot)
        .and_then(|state| maybe_filter_proofs_for_action_name(state, REDEEM_ACTION_NAME))
        .and_then(maybe_parse_redeem_infos_and_put_in_state)
        .and_then(account_for_fees_in_redeem_infos_in_state)
        .and_then(|state| {
            if accrue_fees {
                update_accrued_fees_in_dictionary_and_return_eos_state(state)
            } else {
                info!("✘ Not accruing fees during EOS block reprocessing...");
                Ok(state)
            }
        })
        .and_then(|state| {
            if state.erc20_on_eos_redeem_infos.len() == 0 {
                info!("✔ No redeem infos in state ∴ no ETH transactions to sign!");
                Ok(state)
            } else {
                get_eth_signed_txs(
                    &state.erc20_on_eos_redeem_infos,
                    &state.eth_db_utils.get_erc20_on_eos_smart_contract_address_from_db()?,
                    match maybe_nonce {
                        Some(nonce) => nonce,
                        None => state.eth_db_utils.get_eth_account_nonce_from_db()?,
                    },
                    &state.eth_db_utils.get_eth_chain_id_from_db()?,
                    state.eth_db_utils.get_eth_gas_price_from_db()?,
                    &state.eth_db_utils.get_eth_private_key_from_db()?,
                )
                .and_then(|signed_txs| {
                    #[cfg(feature = "debug")]
                    {
                        debug!("✔ Signed transactions: {:?}", signed_txs);
                    }
                    state.add_eth_signed_txs(signed_txs)
                })
            }
        })
        .and_then(maybe_add_global_sequences_to_processed_list_and_return_state)
        .and_then(|state| {
            if maybe_nonce.is_some() {
                info!("✔ Not incrementing nonce since one was passed in!");
                Ok(state)
            } else {
                maybe_increment_eth_nonce_in_db_and_return_eos_state(state)
            }
        })
        .and_then(end_eos_db_transaction_and_return_state)
        .and_then(|state| {
            info!("✔ Getting EOS output json...");
            let output = serde_json::to_string(&EosOutput {
                eos_latest_block_number: get_latest_eos_block_number(state.db)?,
                eth_signed_transactions: match &state.eth_signed_txs.len() {
                    0 => vec![],
                    _ => get_eth_signed_tx_info_from_eth_txs(
                        &state.eth_signed_txs,
                        &state.erc20_on_eos_redeem_infos,
                        match maybe_nonce {
                            Some(nonce) => nonce,
                            None => state.eth_db_utils.get_eth_account_nonce_from_db()?,
                        },
                        false,
                        state.eth_db_utils.get_any_sender_nonce_from_db()?,
                        state.eth_db_utils.get_latest_eth_block_number()?,
                    )?,
                },
            })?;
            info!("✔ EOS output: {}", output);
            Ok(output)
        })
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
pub fn debug_reprocess_eth_block<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    reprocess_eth_block(db, block_json_string, false)
}

/// # Debug Reprocess ETH Block With Fee Accrual For Stale EOS Transaction
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
    db: D,
    block_json_string: &str,
) -> Result<String> {
    reprocess_eth_block(db, block_json_string, true)
}

/// # Debug Reprocess EOS Block
///
/// This function will take passed in EOS submission material and run it through the simplified
/// submission pipeline, signing and ETH transactions based on valid proofs therein.
///
/// ### NOTES:
///
///  - This function does NOT validate the block to which the proofs (may) pertain.
///
///  - This version of the EOS block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, but it will __not__ accrue those fees on to the total in the
///  dictionary. This is to avoid accounting for fees twice.
///
/// ### BEWARE:
/// This function will incrememnt the ETH nonce in the encrypted database, and so not broadcasting
/// any outputted transactions will result in all future transactions failing. Use only with
/// extreme caution and when you know exactly what you are doing and why.
pub fn debug_reprocess_eos_block<D: DatabaseInterface>(db: D, block_json: &str) -> Result<String> {
    reprocess_eos_block(db, block_json, false, None)
}

/// # Debug Reprocess EOS Block With Fee Accrual
///
/// This function will take passed in EOS submission material and run it through the simplified
/// submission pipeline, signing and ETH transactions based on valid proofs therein.
///
/// ### NOTES:
///
///  - This function does NOT validate the block to which the proofs (may) pertain.
///
///  - This version of the EOS block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, and __will__ accrue those fees on to the total in the
///  dictionary. Only use this is you know what you're doing and why, and make sure you're avoiding
///  accruing the fees twice if the block has already been processed through the non-debug EVM
///  block submission pipeline.
///
/// ### BEWARE:
/// This function will incrememnt the ETH nonce in the encrypted database, and so not broadcasting
/// any outputted transactions will result in all future transactions failing. Use only with
/// extreme caution and when you know exactly what you are doing and why.
pub fn debug_reprocess_eos_block_with_fee_accrual<D: DatabaseInterface>(db: D, block_json: &str) -> Result<String> {
    reprocess_eos_block(db, block_json, true, None)
}

/// # Debug Reprocess EOS Block With Nonce
///
/// This function will take passed in EOS submission material and run it through the simplified
/// submission pipeline, signing and ETH transactions based on valid proofs therein using the
/// passed in nonce. Thus this can be used to replace a transaction.
///
/// ### NOTES:
///
///  - This function does NOT validate the block to which the proofs (may) pertain.
///
///  - This version of the EOS block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, but it will __not__ accrue those fees on to the total in the
///  dictionary. This is to avoid accounting for fees twice.
///
///  - It is assumed that the user of this function understands the nonce ramifications.
pub fn debug_reprocess_eos_block_with_nonce<D: DatabaseInterface>(
    db: D,
    block_json: &str,
    nonce: u64,
) -> Result<String> {
    reprocess_eos_block(db, block_json, false, Some(nonce))
}
