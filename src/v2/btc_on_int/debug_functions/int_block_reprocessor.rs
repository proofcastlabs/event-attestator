use function_name::named;

use crate::{
    btc_on_int::{
        constants::CORE_TYPE,
        int::{
            debug_filter_tx_info_with_no_erc20_transfer_event,
            filter_receipts_for_btc_on_int_redeem_events_in_state,
            get_btc_signed_tx_info_from_btc_txs,
            maybe_sign_btc_txs_and_add_to_state,
            BtcOnIntBtcTxInfos,
            IntOutput,
        },
    },
    chains::{
        btc::increment_btc_account_nonce::maybe_increment_btc_account_nonce_and_return_eth_state,
        eth::{
            eth_database_transactions::end_eth_db_transaction_and_return_state,
            eth_database_utils::EthDbUtilsExt,
            eth_state::EthState,
            eth_submission_material::parse_eth_submission_material_and_put_in_state,
            validate_block_in_state::validate_eth_block_in_state,
        },
    },
    core_type::CoreType,
    debug_functions::validate_debug_command_signature,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

#[named]
fn reprocess_int_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), block_json)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash))
        .and_then(|_| parse_eth_submission_material_and_put_in_state(block_json, EthState::init(db)))
        .and_then(CoreType::check_core_is_initialized_and_return_eth_state)
        .and_then(validate_eth_block_in_state)
        .and_then(filter_receipts_for_btc_on_int_redeem_events_in_state)
        .and_then(|state| {
            state
                .get_eth_submission_material()
                .and_then(|material| {
                    BtcOnIntBtcTxInfos::from_eth_submission_material(
                        material,
                        &state.eth_db_utils.get_btc_on_eth_smart_contract_address_from_db()?,
                    )
                })
                .and_then(|params| state.add_btc_on_int_btc_tx_infos(params))
        })
        .and_then(debug_filter_tx_info_with_no_erc20_transfer_event)
        .and_then(maybe_sign_btc_txs_and_add_to_state)
        .and_then(maybe_increment_btc_account_nonce_and_return_eth_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(|state| {
            info!("✔ Getting INT output json...");
            let output = serde_json::to_string(&IntOutput {
                int_latest_block_number: state.eth_db_utils.get_latest_eth_block_number()?,
                btc_signed_transactions: match state.btc_transactions {
                    Some(txs) => get_btc_signed_tx_info_from_btc_txs(
                        state.btc_db_utils.get_btc_account_nonce_from_db()?,
                        txs,
                        &state.btc_on_int_btc_tx_infos,
                        state.btc_db_utils.get_latest_btc_block_number()?,
                        &state.eth_db_utils.get_btc_on_int_smart_contract_address_from_db()?,
                        &state.btc_db_utils.get_btc_chain_id_from_db()?,
                    )?,
                    None => vec![],
                },
            })?;
            info!("✔ INT Output: {}", output);
            Ok(output)
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Reprocess INT Block
///
/// This function will take a passed in INT block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block
///
/// ### NOTE:
///
///  - This function will increment the core's INT nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, ALL future BTC transactions will
/// fail due to the core having an incorret set of UTXOs!
pub fn debug_reprocess_int_block<D: DatabaseInterface>(db: &D, block_json: &str, signature: &str) -> Result<String> {
    reprocess_int_block(db, block_json, signature)
}
