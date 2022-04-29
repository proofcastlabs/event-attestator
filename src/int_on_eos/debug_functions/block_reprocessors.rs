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
            eth_debug_functions::check_custom_nonce,
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
    int_on_eos::{
        check_core_is_initialized::{
            check_core_is_initialized_and_return_eos_state,
            check_core_is_initialized_and_return_eth_state,
        },
        eos::{
            get_eos_output::{get_tx_infos_from_signed_txs, EosOutput},
            increment_int_nonce::maybe_increment_int_nonce_in_db_and_return_eos_state,
            parse_tx_info::maybe_parse_int_tx_infos_and_put_in_state,
        },
        int::{
            eos_tx_info::IntOnEosEosTxInfos,
            filter_out_zero_tx_infos::filter_out_zero_value_eos_tx_infos_from_state,
            filter_submission_material::filter_submission_material_for_relevant_receipts_in_state,
            get_output_json::get_output_json,
            sign_txs::maybe_sign_eos_txs_and_add_to_eth_state,
        },
    },
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

fn reprocess_int_block<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    info!("✔ Debug reprocessing INT block...");
    parse_eth_submission_material_and_put_in_state(block_json_string, EthState::init(&db))
        .and_then(check_core_is_initialized_and_return_eth_state)
        .and_then(start_eth_db_transaction_and_return_state)
        .and_then(validate_block_in_state)
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
                        let int_db_utils = &EthDbUtils::new(&db);
                        IntOnEosEosTxInfos::from_submission_material(
                            &submission_material,
                            &token_dictionary,
                            &int_db_utils.get_eth_chain_id_from_db()?,
                            &int_db_utils.get_int_on_eos_smart_contract_address_from_db()?,
                            &int_db_utils.get_eth_router_smart_contract_address_from_db()?,
                        )
                    })
                    .and_then(|tx_infos| state.add_int_on_eos_eos_tx_infos(tx_infos))
            }
        })
        .and_then(filter_out_zero_value_eos_tx_infos_from_state)
        .and_then(maybe_sign_eos_txs_and_add_to_eth_state)
        .and_then(maybe_increment_eos_account_nonce_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(get_output_json)
}

fn reprocess_eos_block<D: DatabaseInterface>(db: D, block_json: &str, maybe_nonce: Option<u64>) -> Result<String> {
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
        .and_then(maybe_parse_int_tx_infos_and_put_in_state)
        .and_then(|state| {
            let tx_infos = state.int_on_eos_int_tx_infos.clone();
            if tx_infos.is_empty() {
                info!("✔ No tx infos in state ∴ no INT transactions to sign!");
                Ok(state)
            } else {
                tx_infos
                    .to_signed_txs(
                        match maybe_nonce {
                            Some(nonce) => {
                                info!("✔ Signing txs starting with passed in nonce of {}!", nonce);
                                nonce
                            },
                            None => state.eth_db_utils.get_eth_account_nonce_from_db()?,
                        },
                        state.eth_db_utils.get_eth_gas_price_from_db()?,
                        &state.eth_db_utils.get_eth_chain_id_from_db()?,
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
                maybe_increment_int_nonce_in_db_and_return_eos_state(state)
            }
        })
        .and_then(end_eos_db_transaction_and_return_state)
        .and_then(|state| {
            info!("✔ Getting EOS output json...");
            let txs = state.eth_signed_txs.clone();
            let num_txs = txs.len();
            let output = serde_json::to_string(&EosOutput {
                eos_latest_block_number: state.eos_db_utils.get_latest_eos_block_number()?,
                int_signed_transactions: if num_txs == 0 {
                    vec![]
                } else {
                    get_tx_infos_from_signed_txs(
                        &txs,
                        &state.int_on_eos_int_tx_infos,
                        match maybe_nonce {
                            // NOTE: We increment the passed in nonce ∵ of the way the report nonce is calculated.
                            Some(nonce) => nonce + num_txs as u64,
                            None => state.eth_db_utils.get_eth_account_nonce_from_db()?,
                        },
                        state.eth_db_utils.get_any_sender_nonce_from_db()?,
                        state.eth_db_utils.get_latest_eth_block_number()?,
                    )?
                },
            })?;
            info!("✔ EOS output: {}", output);
            Ok(output)
        })
        .map(prepend_debug_output_marker_to_string)
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
pub fn debug_reprocess_int_block<D: DatabaseInterface>(db: D, block_json_string: &str) -> Result<String> {
    reprocess_int_block(db, block_json_string)
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
/// ### BEWARE:
/// This function will incrememnt the ETH nonce in the encrypted database, and so not broadcasting
/// any outputted transactions will result in all future transactions failing. Use only with
/// extreme caution and when you know exactly what you are doing and why.
pub fn debug_reprocess_eos_block<D: DatabaseInterface>(db: D, block_json: &str) -> Result<String> {
    reprocess_eos_block(db, block_json, None)
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
/// ### BEWARE:
///
/// It is assumed that you know what you're doing nonce-wise with this function!
pub fn debug_reprocess_eos_block_with_nonce<D: DatabaseInterface>(
    db: D,
    block_json: &str,
    nonce: u64,
) -> Result<String> {
    check_custom_nonce(&EthDbUtils::new(&db), nonce).and_then(|_| reprocess_eos_block(db, block_json, Some(nonce)))
}
