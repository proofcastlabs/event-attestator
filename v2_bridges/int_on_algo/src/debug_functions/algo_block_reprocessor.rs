use common::{
    chains::eth::eth_database_utils::EthDbUtilsExt,
    core_type::CoreType,
    traits::DatabaseInterface,
    types::Result,
};
use common_algorand::{
    end_algo_db_transaction_and_return_state,
    maybe_update_latest_block_with_expired_participants_and_return_state,
    parse_algo_submission_material_and_put_in_state,
    remove_all_txs_from_submission_material_in_state,
    AlgoState,
};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;

use crate::{
    algo::{
        add_relevant_validated_txs_to_submission_material_in_state,
        divert_tx_infos_to_safe_address_if_destination_is_router_address,
        divert_tx_infos_to_safe_address_if_destination_is_token_address,
        divert_tx_infos_to_safe_address_if_destination_is_vault_address,
        divert_tx_infos_to_safe_address_if_destination_is_zero_address,
        filter_out_invalid_txs_and_update_in_state,
        filter_out_zero_value_tx_infos_from_state,
        get_int_signed_tx_info_from_algo_txs,
        get_relevant_asset_txs_from_submission_material_and_add_to_state,
        maybe_increment_eth_account_nonce_and_return_algo_state,
        AlgoOutput,
        IntOnAlgoIntTxInfos,
    },
    constants::CORE_TYPE,
    token_dictionary::get_evm_algo_token_dictionary_and_add_to_algo_state,
};

#[named]
fn debug_reprocess_algo_block_maybe_with_nonce<D: DatabaseInterface>(
    db: &D,
    block_json_string: &str,
    maybe_nonce: Option<u64>,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug reprocessing ALGO block...");
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json_string, &maybe_nonce)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| parse_algo_submission_material_and_put_in_state(block_json_string, AlgoState::init(db)))
        .and_then(get_evm_algo_token_dictionary_and_add_to_algo_state)
        .and_then(maybe_update_latest_block_with_expired_participants_and_return_state)
        .and_then(get_relevant_asset_txs_from_submission_material_and_add_to_state)
        .and_then(filter_out_invalid_txs_and_update_in_state)
        .and_then(remove_all_txs_from_submission_material_in_state)
        .and_then(add_relevant_validated_txs_to_submission_material_in_state)
        .and_then(|state| {
            let material = state.get_algo_submission_material()?;
            match material.block.transactions {
                None => {
                    info!("✔ No transactions in canon submission material ∴ no tx info to parse!");
                    Ok(state)
                },
                Some(txs) => {
                    info!(
                        "✔ {} transactions in canon submission material ∴ parsing info...",
                        txs.len()
                    );
                    let dictionary = state.get_evm_algo_token_dictionary()?;
                    IntOnAlgoIntTxInfos::from_algo_txs(
                        &txs,
                        &dictionary,
                        &state.eth_db_utils.get_eth_router_smart_contract_address_from_db()?,
                        &state.algo_db_utils.get_genesis_hash()?,
                        &state.eth_db_utils.get_int_on_algo_smart_contract_address()?,
                    )
                    .and_then(|tx_infos| tx_infos.to_bytes())
                    .map(|bytes| state.add_tx_infos(bytes))
                },
            }
        })
        .and_then(filter_out_zero_value_tx_infos_from_state)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_zero_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_token_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_vault_address)
        .and_then(divert_tx_infos_to_safe_address_if_destination_is_router_address)
        .and_then(|state| {
            if state.tx_infos.is_empty() {
                warn!("✔ No tx infos in state ∴ no INT transactions to sign!");
                Ok(state)
            } else {
                info!("✔ Signing transactions for `IntOnAlgoIntTxInfos`...");
                IntOnAlgoIntTxInfos::from_bytes(&state.tx_infos)
                    .and_then(|tx_infos| {
                        tx_infos.to_eth_signed_txs(
                            match maybe_nonce {
                                Some(nonce) => nonce,
                                None => state.eth_db_utils.get_eth_account_nonce_from_db()?,
                            },
                            &state.eth_db_utils.get_eth_chain_id_from_db()?,
                            state.eth_db_utils.get_eth_gas_price_from_db()?,
                            &state.eth_db_utils.get_eth_private_key_from_db()?,
                        )
                    })
                    .and_then(|signed_txs| {
                        debug!("✔ Signed transactions: {:?}", signed_txs);
                        state.add_eth_signed_txs(signed_txs)
                    })
            }
        })
        .and_then(|state| {
            if maybe_nonce.is_some() {
                info!("✘ Not incrementing nonce because one was passed in!");
                Ok(state)
            } else {
                maybe_increment_eth_account_nonce_and_return_algo_state(state)
            }
        })
        .and_then(end_algo_db_transaction_and_return_state)
        .and_then(|state| {
            info!("✔ Getting ALGO output...");
            let signed_txs = state.eth_signed_txs.clone();
            let tx_infos = IntOnAlgoIntTxInfos::from_bytes(&state.tx_infos)?;
            let output = AlgoOutput {
                algo_latest_block_number: state.algo_db_utils.get_latest_block_number()?,
                int_signed_transactions: if signed_txs.is_empty() {
                    vec![]
                } else {
                    get_int_signed_tx_info_from_algo_txs(
                        &signed_txs,
                        &tx_infos,
                        match maybe_nonce {
                            Some(nonce) => nonce,
                            None => state.eth_db_utils.get_eth_account_nonce_from_db()?,
                        },
                        state.eth_db_utils.get_latest_eth_block_number()?,
                    )?
                },
            };
            Ok(output.to_string())
        })
}

/// # Debug Reprocess ALGO Block
///
/// This function will take ALGO submission material and run it through the
/// submission pipeline, signing any signatures for peg-outs it may find in the block.
///
/// ### NOTES:
///
///  - This function will increment the core's INT nonce by the number of transactions signed.
pub fn debug_reprocess_algo_block<D: DatabaseInterface>(
    db: &D,
    block_json_string: &str,
    signature: &str,
) -> Result<String> {
    debug_reprocess_algo_block_maybe_with_nonce(db, block_json_string, None, signature)
}

/// # Debug Reprocess ALGO Block With Nonce
///
/// This function will take ALGO submission material and run it through the
/// submission pipeline, signing any signatures for peg-outs it may find in the block.
/// This version of the ALGO reprocessor also takes a nonce, which will be used when
/// signing any peg-out transactions. This feature may be used to replace transactions.
///
/// ### NOTES:
///
///  - This function will NOT increment the core's INT nonce, since it uses the one passed in.
///
/// ### BEWARE:
///
/// It is assumed that you know what you're doing nonce-wise with this function!
pub fn debug_reprocess_algo_block_with_nonce<D: DatabaseInterface>(
    db: &D,
    block_json_string: &str,
    nonce: u64,
    signature: &str,
) -> Result<String> {
    debug_reprocess_algo_block_maybe_with_nonce(db, block_json_string, Some(nonce), signature)
}
