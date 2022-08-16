use function_name::named;
pub use serde_json::json;

use crate::{
    chains::{
        eos::{
            add_schedule::maybe_add_new_eos_schedule_to_db_and_return_state,
            eos_database_transactions::end_eos_db_transaction_and_return_state,
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
                maybe_filter_proofs_for_v2_redeem_actions,
            },
            get_enabled_protocol_features::get_enabled_protocol_features_and_add_to_state,
        },
        eth::{
            eth_database_utils::{EthDbUtils, EthDbUtilsExt},
            eth_debug_functions::check_custom_nonce,
        },
    },
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    dictionaries::eos_eth::get_eos_eth_token_dictionary_from_db_and_add_to_eos_state,
    int_on_eos::{
        check_core_is_initialized::check_core_is_initialized_and_return_eos_state,
        constants::CORE_TYPE,
        eos::{
            divert_tx_infos_to_safe_address_if_destination_is_router_address,
            divert_tx_infos_to_safe_address_if_destination_is_token_address,
            divert_tx_infos_to_safe_address_if_destination_is_vault_address,
            divert_tx_infos_to_safe_address_if_destination_is_zero_address,
            get_tx_infos_from_signed_txs,
            maybe_increment_int_nonce_in_db_and_return_eos_state,
            maybe_parse_int_tx_infos_and_put_in_state,
            EosOutput,
        },
    },
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

#[named]
fn reprocess_eos_block<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    maybe_nonce: Option<u64>,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug reprocessing EOS block...");
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json, &maybe_nonce)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| parse_submission_material_and_add_to_state(block_json, EosState::init(db)))
        .and_then(check_core_is_initialized_and_return_eos_state)
        .and_then(get_enabled_protocol_features_and_add_to_state)
        .and_then(get_processed_global_sequences_and_add_to_state)
        .and_then(get_eos_eth_token_dictionary_from_db_and_add_to_eos_state)
        .and_then(maybe_add_new_eos_schedule_to_db_and_return_state)
        .and_then(maybe_filter_duplicate_proofs_from_state)
        .and_then(maybe_filter_out_proofs_for_accounts_not_in_token_dictionary)
        .and_then(maybe_filter_out_action_proof_receipt_mismatches_and_return_state)
        .and_then(maybe_filter_out_invalid_action_receipt_digests)
        .and_then(maybe_filter_out_proofs_with_invalid_merkle_proofs)
        .and_then(maybe_filter_out_proofs_with_wrong_action_mroot)
        .and_then(maybe_filter_proofs_for_v2_redeem_actions)
        .and_then(maybe_parse_int_tx_infos_and_put_in_state)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_vault_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
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
                        state.eth_db_utils.get_latest_eth_block_number()?,
                    )?
                },
            })?;
            info!("✔ EOS output: {}", output);
            Ok(output)
        })
        .map(prepend_debug_output_marker_to_string)
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
pub fn debug_reprocess_eos_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    reprocess_eos_block(db, block_json, None, signature)
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
    db: &D,
    block_json: &str,
    nonce: u64,
    signature: &str,
) -> Result<String> {
    check_custom_nonce(&EthDbUtils::new(db), nonce)
        .and_then(|_| reprocess_eos_block(db, block_json, Some(nonce), signature))
}
