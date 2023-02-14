use common::{
    chains::btc::{
        btc_block::parse_btc_block_and_id_and_put_in_state,
        btc_database_utils::end_btc_db_transaction,
        btc_submission_material::parse_btc_submission_json_and_put_in_state,
        extract_utxos_from_p2sh_txs::maybe_extract_utxos_from_p2sh_txs_and_put_in_state,
        filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
        filter_utxos::filter_out_value_too_low_utxos_from_state,
        get_deposit_info_hash_map::get_deposit_info_hash_map_and_put_in_state,
        save_utxos_to_db::maybe_save_utxos_to_db,
        set_flags::set_any_sender_flag_in_state,
        validate_btc_block_header::validate_btc_block_header_in_state,
        validate_btc_merkle_root::validate_btc_merkle_root,
        validate_btc_proof_of_work::validate_proof_of_work_of_btc_block_in_state,
        BtcState,
    },
    core_type::CoreType,
    fees::fee_database_utils::FeeDatabaseUtils,
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use common_eth::{EthDbUtils, EthDbUtilsExt, EthSigningParams, EthTransactions};
use common_eth_debug::check_custom_nonce;
use function_name::named;

use crate::{
    btc::{
        get_eth_signed_tx_info_from_eth_txs,
        get_eth_signed_txs,
        maybe_account_for_minting_fees,
        maybe_divert_txs_to_safe_address_if_destination_is_token_address,
        maybe_filter_out_value_too_low_btc_on_eth_eth_tx_infos_in_state,
        maybe_increment_nonce_in_db,
        parse_eth_tx_infos_from_p2sh_deposits_and_add_to_state,
        subtract_fees_from_eth_tx_infos,
        BtcOnEthEthTxInfos,
    },
    constants::CORE_TYPE,
};

#[named]
fn reprocess_btc_block<D: DatabaseInterface>(
    db: &D,
    btc_submission_material_json: &str,
    accrue_fees: bool,
    maybe_nonce: Option<u64>,
    signature: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), btc_submission_material_json, &accrue_fees)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| parse_btc_submission_json_and_put_in_state(btc_submission_material_json, BtcState::init(db)))
        .and_then(set_any_sender_flag_in_state)
        .and_then(parse_btc_block_and_id_and_put_in_state)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(parse_eth_tx_infos_from_p2sh_deposits_and_add_to_state)
        .and_then(maybe_extract_utxos_from_p2sh_txs_and_put_in_state)
        .and_then(filter_out_value_too_low_utxos_from_state)
        .and_then(maybe_save_utxos_to_db)
        .and_then(maybe_filter_out_value_too_low_btc_on_eth_eth_tx_infos_in_state)
        .and_then(maybe_divert_txs_to_safe_address_if_destination_is_token_address)
        .and_then(|state| {
            if accrue_fees {
                maybe_account_for_minting_fees(state)
            } else {
                info!("✘ Not accruing fees during BTC block reprocessing...");
                subtract_fees_from_eth_tx_infos(
                    &BtcOnEthEthTxInfos::from_bytes(&state.tx_infos)?,
                    FeeDatabaseUtils::new_for_btc_on_eth().get_peg_in_basis_points_from_db(state.db)?,
                )
                .and_then(|tx_infos| tx_infos.to_bytes())
                .map(|bytes| state.add_tx_infos(bytes))
            }
        })
        .and_then(|state| {
            let eth_db_utils = EthDbUtils::new(state.db);
            get_eth_signed_txs(
                &EthSigningParams {
                    gas_price: eth_db_utils.get_eth_gas_price_from_db()?,
                    chain_id: eth_db_utils.get_eth_chain_id_from_db()?,
                    eth_private_key: eth_db_utils.get_eth_private_key_from_db()?,
                    eth_account_nonce: match maybe_nonce {
                        Some(nonce) => {
                            info!("✔ Signing txs starting with passed in nonce of {}!", nonce);
                            nonce
                        },
                        None => eth_db_utils.get_eth_account_nonce_from_db()?,
                    },
                    smart_contract_address: eth_db_utils.get_btc_on_eth_smart_contract_address_from_db()?,
                },
                &BtcOnEthEthTxInfos::from_bytes(&state.tx_infos)?,
                &state.btc_db_utils.get_btc_chain_id_from_db()?,
            )
            .and_then(|signed_txs| signed_txs.to_bytes())
            .map(|bytes| state.add_eth_signed_txs(bytes))
        })
        .and_then(|state| {
            if maybe_nonce.is_some() {
                info!("✔ Not incrementing nonce since one was passed in!");
                Ok(state)
            } else {
                maybe_increment_nonce_in_db(state)
            }
        })
        .and_then(|state| {
            let txs = EthTransactions::from_bytes(&state.eth_signed_txs)?;
            let num_txs = txs.len();
            let eth_db_utils = EthDbUtils::new(state.db);
            let signatures = serde_json::to_string(&if num_txs == 0 {
                Ok(vec![])
            } else {
                get_eth_signed_tx_info_from_eth_txs(
                    &txs,
                    &BtcOnEthEthTxInfos::from_bytes(&state.tx_infos)?,
                    match maybe_nonce {
                        // NOTE: We increment the passed in nonce ∵ of the way the report nonce is calculated.
                        Some(nonce) => nonce + num_txs as u64,
                        None => eth_db_utils.get_eth_account_nonce_from_db()?,
                    },
                    state.use_any_sender_tx_type(),
                    eth_db_utils.get_any_sender_nonce_from_db()?,
                )
            }?)?;
            info!("✔ ETH Signatures: {}", signatures);
            state.add_output_json_string(signatures)
        })
        .and_then(end_btc_db_transaction)
        .map(|state| match state.output_json_string {
            None => "✘ No signatures signed ∴ no output!".to_string(),
            Some(output) => output,
        })
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Reprocess BTC Block
///
/// This function will take a passed in ETH block submission material and run it through the
/// submission pipeline, signing any signatures for pegins it may find in the block
///
/// ### NOTE:
///
///  - This does not yet work with AnySender type transactions.
///
///  - This version of the BTC block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, but it will __not__ accrue those fees on to the total in the
///  dictionary. This is to avoid accounting for fees twice.
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, future ETH transactions will
/// fail due to an incorrect nonce!
pub fn debug_reprocess_btc_block<D: DatabaseInterface>(
    db: &D,
    btc_submission_material_json: &str,
    signature: &str,
) -> Result<String> {
    reprocess_btc_block(db, btc_submission_material_json, false, None, signature)
}

/// # Debug Reprocess BTC Block With Nonce
///
/// This function will take a passed in BTC block submission material and run it through the
/// submission pipeline, signing any signatures for pegins it may find in the block, using the
/// passed in nonce. Thus it may be used to replace transactions.
///
/// ### NOTE:
///
///  - This does not yet work with AnySender type transactions.
///
///  - This version of the BTC block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, but it will __not__ accrue those fees on to the total in the
///  dictionary. This is to avoid accounting for fees twice.
///
/// ### BEWARE:
///
/// It is assumed that you know what you're doing nonce-wise with this function!
pub fn debug_reprocess_btc_block_with_nonce<D: DatabaseInterface>(
    db: &D,
    btc_submission_material_json: &str,
    nonce: u64,
    signature: &str,
) -> Result<String> {
    check_custom_nonce(&EthDbUtils::new(db), nonce)
        .and_then(|_| reprocess_btc_block(db, btc_submission_material_json, false, Some(nonce), signature))
}

/// # Debug Reprocess BTC Block With Fee Accrual
///
/// This function will take a passed in ETH block submission material and run it through the
/// submission pipeline, signing any signatures for pegins it may find in the block
///
/// ### NOTE:
///
///  - This does not yet work with AnySender type transactions.
///
///  - This version of the BTC block reprocessor __will__ deduct fees from any transaction info(s) it
///  parses from the submitted block, and __will__ accrue those fees on to the total. Only use this if
///  you know what you're doing and why, and make sure you're avoiding accruing the fees twice if the
///  block has already been processed through the non-debug BTC block submission pipeline.
///
/// ### BEWARE:
/// If you don't broadcast the transaction outputted from this function, future ETH transactions will
/// fail due to an incorrect nonce!
pub fn debug_reprocess_btc_block_with_fee_accrual<D: DatabaseInterface>(
    db: &D,
    btc_submission_material_json: &str,
    signature: &str,
) -> Result<String> {
    reprocess_btc_block(db, btc_submission_material_json, true, None, signature)
}
