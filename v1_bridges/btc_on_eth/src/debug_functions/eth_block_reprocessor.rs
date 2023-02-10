use common::{
    core_type::CoreType,
    fees::fee_database_utils::FeeDatabaseUtils,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use common_eth::{
    end_eth_db_transaction_and_return_state,
    parse_eth_submission_material_and_put_in_state,
    validate_eth_block_in_state,
    EthDbUtilsExt,
    EthState,
};
use function_name::named;

use crate::{
    constants::CORE_TYPE,
    eth::{
        filter_receipts_for_btc_on_eth_redeem_events_in_state,
        get_btc_signed_tx_info_from_btc_txs,
        maybe_account_for_fees,
        maybe_create_btc_txs_and_add_to_state,
        maybe_increment_btc_nonce_in_db_and_return_state,
        subtract_fees_from_btc_tx_infos,
        BtcOnEthBtcTxInfos,
        EthOutput,
    },
};

#[named]
fn reprocess_eth_block<D: DatabaseInterface>(
    db: &D,
    eth_block_json: &str,
    accrue_fees: bool,
    signature: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), eth_block_json, &accrue_fees)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| parse_eth_submission_material_and_put_in_state(eth_block_json, EthState::init(db)))
        .and_then(validate_eth_block_in_state)
        .and_then(filter_receipts_for_btc_on_eth_redeem_events_in_state)
        .and_then(|state| {
            state
                .get_eth_submission_material()
                .and_then(|material| {
                    BtcOnEthBtcTxInfos::from_eth_submission_material(
                        material,
                        &state.eth_db_utils.get_btc_on_eth_smart_contract_address_from_db()?,
                    )
                })
                .and_then(|params| params.to_bytes())
                .map(|bytes| state.add_tx_infos(bytes))
        })
        .and_then(|state| {
            if accrue_fees {
                maybe_account_for_fees(state)
            } else {
                info!("✘ Not accruing fees during ETH block reprocessing...");
                BtcOnEthBtcTxInfos::from_bytes(&state.tx_infos)
                    .and_then(|tx_infos| {
                        subtract_fees_from_btc_tx_infos(
                            &tx_infos,
                            FeeDatabaseUtils::new_for_btc_on_eth().get_peg_out_basis_points_from_db(state.db)?,
                        )
                    })
                    .and_then(|infos| infos.to_bytes())
                    .map(|bytes| state.add_tx_infos(bytes))
            }
        })
        .and_then(maybe_create_btc_txs_and_add_to_state)
        .and_then(maybe_increment_btc_nonce_in_db_and_return_state)
        .and_then(end_eth_db_transaction_and_return_state)
        .and_then(|state| {
            info!("✔ Getting ETH output json...");
            let output = serde_json::to_string(&EthOutput {
                eth_latest_block_number: state.eth_db_utils.get_latest_eth_block_number()?,
                btc_signed_transactions: match state.btc_transactions {
                    Some(txs) => get_btc_signed_tx_info_from_btc_txs(
                        state.btc_db_utils.get_btc_account_nonce_from_db()?,
                        txs,
                        &BtcOnEthBtcTxInfos::from_bytes(&state.tx_infos)?,
                    )?,
                    None => vec![],
                },
            })?;
            info!("✔ ETH Output: {}", output);
            Ok(output)
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Reprocess ETH Block
///
/// This function will take a passed in ETH block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block
///
/// ### NOTE:
///
///  - This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
///  - This version of the ETH block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, but it will __not__ accrue those fees on to the total in the
///  dictionary. This is to avoid accounting for fees twice.
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, ALL future BTC transactions will
/// fail due to the core having an incorret set of UTXOs!
pub fn debug_reprocess_eth_block<D: DatabaseInterface>(
    db: &D,
    eth_block_json: &str,
    signature: &str,
) -> Result<String> {
    reprocess_eth_block(db, eth_block_json, false, signature)
}

/// # Debug Reprocess ETH Block With Fee Accrual
///
/// This function will take a passed in ETH block submission material and run it through the
/// submission pipeline, signing any signatures for pegouts it may find in the block
///
/// ### NOTE:
///
///  - This function will increment the core's ETH nonce, meaning the outputted reports will have a
/// gap in their report IDs!
///
///  - This version of the ETH block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, and __will__ accrue those fees on to the total. Only use this if
///  you know what you're doing and why, and make sure you're avoiding accruing the fees twice if the
///  block has already been processed through the non-debug BTC block submission pipeline.
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, ALL future BTC transactions will
/// fail due to the core having an incorret set of UTXOs!
pub fn debug_reprocess_eth_block_with_fee_accrual<D: DatabaseInterface>(
    db: &D,
    eth_block_json: &str,
    signature: &str,
) -> Result<String> {
    reprocess_eth_block(db, eth_block_json, true, signature)
}
