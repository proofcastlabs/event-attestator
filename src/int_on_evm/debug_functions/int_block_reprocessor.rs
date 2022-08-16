use function_name::named;

use crate::{
    chains::eth::{
        eth_database_transactions::end_eth_db_transaction_and_return_state,
        eth_database_utils::{EthDbUtilsExt, EvmDbUtils},
        eth_debug_functions::check_custom_nonce,
        eth_state::EthState,
        eth_submission_material::parse_eth_submission_material_and_put_in_state,
        increment_evm_account_nonce::maybe_increment_evm_account_nonce_and_return_eth_state,
        validate_block_in_state::validate_block_in_state,
        validate_receipts_in_state::validate_receipts_in_state,
    },
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    dictionaries::eth_evm::{get_eth_evm_token_dictionary_from_db_and_add_to_eth_state, EthEvmTokenDictionary},
    int_on_evm::{
        check_core_is_initialized::check_core_is_initialized_and_return_eth_state,
        constants::CORE_TYPE,
        int::{
            account_for_fees_in_evm_tx_infos_in_state,
            debug_filter_tx_info_with_no_erc20_transfer_event,
            divert_tx_infos_to_safe_address_if_destination_is_router_address,
            divert_tx_infos_to_safe_address_if_destination_is_token_address,
            divert_tx_infos_to_safe_address_if_destination_is_vault_address,
            divert_tx_infos_to_safe_address_if_destination_is_zero_address,
            filter_out_zero_value_evm_tx_infos_from_state,
            filter_submission_material_for_peg_in_events_in_state,
            get_evm_signed_tx_info_from_int_txs,
            update_accrued_fees_in_dictionary_and_return_eth_state,
            IntOnEvmEvmTxInfos,
            IntOutput,
        },
    },
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

#[named]
fn reprocess_int_block<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    accrue_fees: bool,
    maybe_nonce: Option<u64>,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug reprocessing INT block...");
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json, &accrue_fees, &maybe_nonce)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| parse_eth_submission_material_and_put_in_state(block_json, EthState::init(db)))
        .and_then(check_core_is_initialized_and_return_eth_state)
        .and_then(validate_block_in_state)
        .and_then(validate_receipts_in_state)
        .and_then(get_eth_evm_token_dictionary_from_db_and_add_to_eth_state)
        .and_then(filter_submission_material_for_peg_in_events_in_state)
        .and_then(|state| {
            state
                .get_eth_submission_material()
                .and_then(|material| {
                    IntOnEvmEvmTxInfos::from_submission_material(
                        material,
                        &state.eth_db_utils.get_erc20_on_evm_smart_contract_address_from_db()?,
                        &EthEvmTokenDictionary::get_from_db(state.db)?,
                        &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                    )
                })
                .and_then(|params| state.add_int_on_evm_evm_tx_infos(params))
        })
        .and_then(filter_out_zero_value_evm_tx_infos_from_state)
        .and_then(debug_filter_tx_info_with_no_erc20_transfer_event)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_vault_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(account_for_fees_in_evm_tx_infos_in_state)
        .and_then(|state| {
            if accrue_fees {
                update_accrued_fees_in_dictionary_and_return_eth_state(state)
            } else {
                info!("✘ Not accruing fees during INT block reprocessing...");
                Ok(state)
            }
        })
        //.and_then(maybe_divert_evm_txs_to_safe_address_if_destination_is_token_address)
        .and_then(|state| {
            if state.int_on_evm_evm_tx_infos.is_empty() {
                info!("✔ No tx infos in state ∴ no INT transactions to sign!");
                Ok(state)
            } else {
                let chain_id = state.evm_db_utils.get_eth_chain_id_from_db()?;
                state
                    .int_on_evm_evm_tx_infos
                    .to_evm_signed_txs(
                        match maybe_nonce {
                            Some(nonce) => {
                                info!("✔ Signing txs starting with passed in nonce of {}!", nonce);
                                nonce
                            },
                            None => state.evm_db_utils.get_eth_account_nonce_from_db()?,
                        },
                        &chain_id,
                        chain_id.get_erc777_mint_with_data_gas_limit(),
                        state.evm_db_utils.get_eth_gas_price_from_db()?,
                        &state.evm_db_utils.get_eth_private_key_from_db()?,
                        &EthEvmTokenDictionary::get_from_db(state.db)?,
                    )
                    .and_then(|signed_txs| {
                        #[cfg(feature = "debug")]
                        {
                            debug!("✔ Signed transactions: {:?}", signed_txs);
                        }
                        state.add_int_on_evm_evm_signed_txs(signed_txs)
                    })
            }
        })
        .and_then(|state| {
            if maybe_nonce.is_some() {
                info!("✔ Not incrementing nonce since one was passed in!");
                Ok(state)
            } else {
                maybe_increment_evm_account_nonce_and_return_eth_state(state)
            }
        })
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(|state| {
            info!("✔ Getting INT output json...");
            let txs = state.int_on_evm_evm_signed_txs.clone();
            let num_txs = txs.len();
            let output = serde_json::to_string(&IntOutput {
                int_latest_block_number: state.eth_db_utils.get_latest_eth_block_number()?,
                evm_signed_transactions: if num_txs == 0 {
                    vec![]
                } else {
                    let use_any_sender_tx = false;
                    get_evm_signed_tx_info_from_int_txs(
                        &txs,
                        &state.int_on_evm_evm_tx_infos,
                        match maybe_nonce {
                            // NOTE: We inrement the passed in nonce ∵ of the way the report nonce is calculated.
                            Some(nonce) => nonce + num_txs as u64,
                            None => state.evm_db_utils.get_eth_account_nonce_from_db()?,
                        },
                        use_any_sender_tx,
                        state.evm_db_utils.get_any_sender_nonce_from_db()?,
                        state.evm_db_utils.get_latest_eth_block_number()?,
                        &EthEvmTokenDictionary::get_from_db(state.db)?,
                    )?
                },
            })?;
            info!("✔ Reprocess INT block output: {}", output);
            Ok(output)
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Reprocess INT Block
///
/// This function will take a passed in INT block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block
///
/// ### NOTES:
///  - This function will increment the core's INT nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
///  - This version of the INT block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, but it will __not__ accrue those fees on to the total in the
///  dictionary. This is to avoid accounting for fees twice.
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, ALL future INT transactions will
/// fail due to the core having an incorret nonce!
pub fn debug_reprocess_int_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    reprocess_int_block(db, block_json, false, None, signature)
}

/// # Debug Reprocess INT Block With Nonce
///
/// This function will take a passed in INT block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block, using the
/// passed in nonce for those signatures. Thus it may be used to replace a transaction.
///
/// ### NOTES:
///  - This function will NOT increment the core's INT nonce if one is passed in.
///
///  - This version of the INT block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, but it will __not__ accrue those fees on to the total in the
///  dictionary. This is to avoid accounting for fees twice.
///
/// ### BEWARE:
///
/// It is assumed that you know what you're doing nonce-wise with this function!
pub fn debug_reprocess_int_block_with_nonce<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    nonce: u64,
    signature: &str,
) -> Result<String> {
    check_custom_nonce(&EvmDbUtils::new(db), nonce)
        .and_then(|_| reprocess_int_block(db, block_json, false, Some(nonce), signature))
}

/// # Debug Reprocess INT Block With Fee Accrual
///
/// This function will take a passed in INT block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block
///
/// ### NOTES:
///
///  - This function will increment the core's INT nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
///  - This version of the INT block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, and __will__ accrue those fees on to the total in the
///  dictionary. Only use this is you know what you're doing and why, and make sure you're avoiding
///  accruing the fees twice if the block has already been processed through the non-debug INT
///  block submission pipeline.
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, ALL future INT transactions will
/// fail due to the core having an incorret nonce!
pub fn debug_reprocess_int_block_with_fee_accrual<D: DatabaseInterface>(
    db: &D,
    block_json: &str,
    signature: &str,
) -> Result<String> {
    reprocess_int_block(db, block_json, true, None, signature)
}
