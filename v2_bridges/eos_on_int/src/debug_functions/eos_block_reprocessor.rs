use common::{traits::DatabaseInterface, types::Result, utils::prepend_debug_output_marker_to_string, CoreType};
use common_debug_signers::validate_debug_command_signature;
use common_eos::{
    end_eos_db_transaction_and_return_state,
    get_enabled_protocol_features_and_add_to_state,
    get_processed_global_sequences_and_add_to_state,
    maybe_add_global_sequences_to_processed_list_and_return_state,
    maybe_add_new_eos_schedule_to_db_and_return_state,
    maybe_filter_duplicate_proofs_from_state,
    maybe_filter_out_action_proof_receipt_mismatches_and_return_state,
    maybe_filter_out_invalid_action_receipt_digests,
    maybe_filter_out_proofs_for_wrong_eos_account_name,
    maybe_filter_out_proofs_with_invalid_merkle_proofs,
    maybe_filter_out_proofs_with_wrong_action_mroot,
    maybe_filter_proofs_for_v1_peg_in_actions,
    parse_submission_material_and_add_to_state,
    EosState,
};
use common_eth::{EthDbUtils, EthDbUtilsExt, EthTransactions};
use common_eth_debug::check_custom_nonce;
use function_name::named;

use crate::{
    constants::CORE_TYPE,
    eos::{
        divert_tx_infos_to_safe_address_if_destination_is_router_address,
        divert_tx_infos_to_safe_address_if_destination_is_token_address,
        divert_tx_infos_to_safe_address_if_destination_is_zero_address,
        get_int_signed_tx_info_from_txs,
        maybe_filter_out_value_too_low_txs_from_state,
        maybe_increment_int_nonce_in_db_and_return_eos_state,
        maybe_parse_eos_on_int_int_tx_infos_and_put_in_state,
        EosOnIntIntTxInfos,
        EosOutput,
    },
};

#[named]
fn reprocess_eos_block<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    maybe_nonce: Option<u64>,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug reprocessing EOS block...");
    let eth_db_utils = EthDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json, &maybe_nonce)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| parse_submission_material_and_add_to_state(block_json, EosState::init(db)))
        .and_then(get_enabled_protocol_features_and_add_to_state)
        .and_then(get_processed_global_sequences_and_add_to_state)
        .and_then(|state| state.get_eos_eth_token_dictionary_and_add_to_state())
        .and_then(maybe_add_new_eos_schedule_to_db_and_return_state)
        .and_then(maybe_filter_duplicate_proofs_from_state)
        .and_then(maybe_filter_out_action_proof_receipt_mismatches_and_return_state)
        .and_then(maybe_filter_out_proofs_for_wrong_eos_account_name)
        .and_then(maybe_filter_out_invalid_action_receipt_digests)
        .and_then(maybe_filter_out_proofs_with_invalid_merkle_proofs)
        .and_then(maybe_filter_out_proofs_with_wrong_action_mroot)
        .and_then(maybe_filter_proofs_for_v1_peg_in_actions)
        .and_then(maybe_parse_eos_on_int_int_tx_infos_and_put_in_state)
        .and_then(maybe_filter_out_value_too_low_txs_from_state)
        .and_then(maybe_add_global_sequences_to_processed_list_and_return_state)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(|state| {
            // NOTE: If we're _re_ processing, we don't need to add the global sequence(s) since
            // they'll have been added the first time this block was processed.
            let tx_infos = EosOnIntIntTxInfos::from_bytes(&state.tx_infos)?;
            if tx_infos.is_empty() {
                info!("✔ No EOS tx info in state ∴ no INT transactions to sign!");
                Ok(state)
            } else {
                tx_infos
                    .to_eth_signed_txs(
                        match maybe_nonce {
                            Some(nonce) => {
                                info!("✔ Signing txs starting with passed in nonce of {}!", nonce);
                                nonce
                            },
                            None => eth_db_utils.get_eth_account_nonce_from_db()?,
                        },
                        &eth_db_utils.get_eth_chain_id_from_db()?,
                        eth_db_utils.get_eth_gas_price_from_db()?,
                        &eth_db_utils.get_eth_private_key_from_db()?,
                    )
                    .and_then(|signed_txs| {
                        debug!("✔ Signed transactions: {:?}", signed_txs);
                        signed_txs.to_bytes()
                    })
                    .map(|bytes| state.add_eth_signed_txs(bytes))
            }
        })
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
                    let txs = EthTransactions::from_bytes(&state.eth_signed_txs)?;
                    get_int_signed_tx_info_from_txs(
                        &txs,
                        &EosOnIntIntTxInfos::from_bytes(&state.tx_infos)?,
                        match maybe_nonce {
                            // NOTE: We increment the passed in nonce ∵ of the way the report nonce is calculated.
                            Some(nonce) => nonce + num_txs as u64,
                            None => eth_db_utils.get_eth_account_nonce_from_db()?,
                        },
                        eth_db_utils.get_latest_eth_block_number()?,
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
/// submission pipeline, signing and INT transactions based on valid proofs therein.
///
/// ### NOTES:
///
///  - This function does NOT validate the block to which the proofs (may) pertain.
///
/// ### BEWARE:
///
/// This function will incrememnt the INT nonce in the encrypted database, and so not broadcasting
/// any outputted transactions will result in all future transactions failing. Use only with
/// extreme caution and when you know exactly what you are doing and why.
pub fn debug_reprocess_eos_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    reprocess_eos_block(db, block_json, None, signature)
}

/// # Debug Reprocess EOS Block With Nonce
///
/// This function will take passed in EOS submission material and run it through the simplified
/// submission pipeline, signing and INT transactions based on valid proofs therein using the
/// passed in nonce. Thus it can be used to replace transactions.
///
/// ### NOTES:
///
///  - This function does NOT validate the block to which the proofs (may) pertain.
///
/// ### Beware
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
