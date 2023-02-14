use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};

use crate::{
    add_btc_block_to_db::maybe_add_btc_block_to_db,
    btc_constants::BTC_CORE_IS_INITIALIZED_JSON,
    btc_database_utils::{end_btc_db_transaction, start_btc_db_transaction},
    btc_submission_material::parse_submission_material_and_put_in_state,
    core_initialization::{
        btc_init_utils::{
            initialize_utxo_balance_and_return_state,
            put_btc_account_nonce_in_db_and_return_state,
            put_btc_fee_in_db_and_return_state,
            put_btc_network_in_db_and_return_state,
            put_btc_tail_block_hash_in_db_and_return_state,
            put_canon_to_tip_length_in_db_and_return_state,
            put_difficulty_threshold_in_db,
        },
        generate_and_store_btc_keys::generate_and_store_btc_keys_and_return_state,
        get_btc_init_output_json::get_btc_init_output_json,
    },
    get_btc_block_in_db_format::create_btc_block_in_db_format_and_put_in_state,
    set_btc_anchor_block_hash::maybe_set_btc_anchor_block_hash,
    set_btc_canon_block_hash::maybe_set_btc_canon_block_hash,
    set_btc_latest_block_hash::maybe_set_btc_latest_block_hash,
    validate_btc_block_header::validate_btc_block_header_in_state,
    validate_btc_difficulty::validate_difficulty_of_btc_block_in_state,
    validate_btc_merkle_root::validate_btc_merkle_root,
    validate_btc_proof_of_work::validate_proof_of_work_of_btc_block_in_state,
    BtcState,
};

pub fn init_btc_core<D: DatabaseInterface>(
    state: BtcState<D>,
    block_json_string: &str,
    fee: u64,
    difficulty: u64,
    network: &str,
    canon_to_tip_length: u64,
) -> Result<String> {
    info!("✔ Initializing enclave for BTC...");
    start_btc_db_transaction(state)
        .and_then(|state| put_difficulty_threshold_in_db(difficulty, state))
        .and_then(|state| put_btc_network_in_db_and_return_state(network, state))
        .and_then(|state| put_btc_fee_in_db_and_return_state(fee, state))
        .and_then(|state| parse_submission_material_and_put_in_state(block_json_string, state))
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_difficulty_of_btc_block_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(|state| put_canon_to_tip_length_in_db_and_return_state(canon_to_tip_length, state))
        .and_then(maybe_set_btc_anchor_block_hash)
        .and_then(maybe_set_btc_latest_block_hash)
        .and_then(maybe_set_btc_canon_block_hash)
        .and_then(put_btc_tail_block_hash_in_db_and_return_state)
        .and_then(create_btc_block_in_db_format_and_put_in_state)
        .and_then(maybe_add_btc_block_to_db)
        .and_then(put_btc_account_nonce_in_db_and_return_state)
        .and_then(initialize_utxo_balance_and_return_state)
        .and_then(|state| generate_and_store_btc_keys_and_return_state(network, state))
        .and_then(|state| {
            // NOTE: BTC is ALWAYS native, since it cannot host pTokens.
            CoreType::initialize_native_core(state.btc_db_utils.get_db())?;
            Ok(state)
        })
        .and_then(end_btc_db_transaction)
        .and_then(get_btc_init_output_json)
}

pub fn maybe_initialize_btc_core<D: DatabaseInterface>(
    db: &D,
    block_json_string: &str,
    fee: u64,
    difficulty: u64,
    network: &str,
    canon_to_tip_length: u64,
) -> Result<String> {
    info!("✔ Maybe initializing BTC core...");
    let state = BtcState::init(db);
    // NOTE: BTC is ALWAYS native, since it cannot host pTokens.
    if CoreType::native_core_is_initialized(db) {
        Ok(BTC_CORE_IS_INITIALIZED_JSON.to_string())
    } else {
        init_btc_core(state, block_json_string, fee, difficulty, network, canon_to_tip_length)
    }
}
