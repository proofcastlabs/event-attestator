use crate::{
    types::Result,
    traits::DatabaseInterface,
    check_debug_mode::check_debug_mode,
    debug_database_utils::{
        get_key_from_db,
        set_key_in_db_to_value,
    },
    chains::{
        eos::eos_constants::EOS_PRIVATE_KEY_DB_KEY as EOS_KEY,
        btc::{
            btc_constants::BTC_PRIVATE_KEY_DB_KEY as BTC_KEY,
            utxo_manager::{
                debug_utxo_utils::clear_all_utxos,
                utxo_utils::get_all_utxos_as_json_string,
            },
        },
    },
    btc_on_eos::{
        check_core_is_initialized::{
            check_core_is_initialized,
            check_core_is_initialized_and_return_btc_state,
        },
        btc::{
            btc_state::BtcState,
            sign_transactions::get_signed_txs,
            btc_database_utils::start_btc_db_transaction,
            get_btc_output_json::get_btc_output_as_string,
            btc_database_utils::get_btc_latest_block_from_db,
            validate_btc_merkle_root::validate_btc_merkle_root,
            filter_minting_params::maybe_filter_minting_params_in_state,
            validate_btc_block_header::validate_btc_block_header_in_state,
            filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
            validate_btc_difficulty::validate_difficulty_of_btc_block_in_state,
            filter_too_short_names::maybe_filter_name_too_short_params_in_state,
            get_btc_output_json::{
                    BtcOutput,
                    get_eos_signed_tx_info_from_eth_txs,
            },
            get_deposit_info_hash_map::{
                get_deposit_info_hash_map_and_put_in_state,
            },
            parse_submission_material::{
                parse_submission_material_and_put_in_state,
            },
            validate_btc_proof_of_work::{
                validate_proof_of_work_of_btc_block_in_state,
            },
            get_btc_block_in_db_format::{
                create_btc_block_in_db_format_and_put_in_state
            },
            parse_minting_params_from_p2sh_deposits::{
                parse_minting_params_from_p2sh_deposits_and_add_to_state,
            },
	},
        eos::{
            eos_crypto::eos_private_key::EosPrivateKey,
            parse_eos_schedule::parse_schedule_string_to_schedule,
            eos_database_utils::{
                get_eos_chain_id_from_db,
                get_eos_account_name_string_from_db,
            },
            eos_database_utils::{
                put_eos_schedule_in_db,
		get_eos_account_nonce_from_db
            },
            initialize_eos::eos_init_utils::{
                EosInitJson,
                put_eos_latest_block_info_in_db,
                generate_and_put_incremerkle_in_db,
            },
        },
    },
};

pub fn debug_reprocess_btc_block_for_stale_eos_tx<D>(
    db: D,
    block_json_string: String
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Reprocessing BTC block to core...");
    parse_submission_material_and_put_in_state(
        block_json_string,
        BtcState::init(db),
    )
        .and_then(check_core_is_initialized_and_return_btc_state)
        .and_then(start_btc_db_transaction)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_difficulty_of_btc_block_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(parse_minting_params_from_p2sh_deposits_and_add_to_state)
        .and_then(maybe_filter_minting_params_in_state)
        .and_then(maybe_filter_name_too_short_params_in_state)
        .and_then(create_btc_block_in_db_format_and_put_in_state)
        .and_then(|state| {
	    info!("✔ Maybe signing reprocessed minting txs...");
	    get_signed_txs(
		state.ref_block_num,
		state.ref_block_prefix,
		&get_eos_chain_id_from_db(&state.db)?,
		&EosPrivateKey::get_from_db(&state.db)?,
		&get_eos_account_name_string_from_db(&state.db)?,
		&state.minting_params,
	    )
		.and_then(|signed_txs| {
			info!("✔ EOS Signed Txs: {:?}", signed_txs);
			state.add_signed_txs(signed_txs)
		})
	})
        .and_then(|state| {
	    info!("✔ Getting BTC output json and putting in state...");
	    Ok(serde_json::to_string(
		&BtcOutput {
		    btc_latest_block_number: get_btc_latest_block_from_db(
                         &state.db
                     )?.height,
		    eos_signed_transactions: match &state.signed_txs.len() {
			0 => vec![],
			_ =>
			    get_eos_signed_tx_info_from_eth_txs(
				&state.signed_txs,
				&state.minting_params,
				get_eos_account_nonce_from_db(&state.db)?,
			    )?,
		    }
		}
	    )?)
		.and_then(|output| state.add_output_json_string(output))
	})
	.and_then(get_btc_output_as_string)
}

pub fn debug_update_incremerkle<D>(
    db: &D,
    eos_init_json: String,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Debug updating blockroot merkle...");
    let init_json = EosInitJson::from_json_string(&eos_init_json)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(db))
        .and_then(|_| put_eos_latest_block_info_in_db(db, &init_json.block))
        .and_then(|_|
            generate_and_put_incremerkle_in_db(db, &init_json.blockroot_merkle)
        )
        .and_then(|_|
            Ok("{debug_update_blockroot_merkle_success:true}".to_string())
        )
}

pub fn debug_clear_all_utxos<D>(
    db: &D,
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Debug clearing all UTXOs...");
    check_core_is_initialized(db)
        .and_then(|_| clear_all_utxos(db))
}

pub fn debug_add_new_eos_schedule<D>(
    db: D,
    schedule_json: String
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Debug adding new EOS schedule...");
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| db.start_transaction())
        .and_then(|_| parse_schedule_string_to_schedule(&schedule_json))
        .and_then(|schedule| put_eos_schedule_in_db(&db, &schedule))
        .and_then(|_| db.end_transaction())
        .map(|_| "{debug_adding_eos_schedule_succeeded:true}".to_string())
}

pub fn debug_set_key_in_db_to_value<D>(
    db: D,
    key: String,
    value: String
) -> Result<String>
    where D: DatabaseInterface
{
    check_core_is_initialized(&db)
        .and_then(|_| set_key_in_db_to_value(db, key, value))
}

pub fn debug_get_key_from_db<D>(
    db: D,
    key: String
) -> Result<String>
    where D: DatabaseInterface
{
    let key_bytes = hex::decode(&key)?;
    check_core_is_initialized(&db)
        .and_then(|_| {
            if key_bytes == EOS_KEY.to_vec() || key_bytes == BTC_KEY.to_vec() {
                get_key_from_db(db, key, Some(255))
            } else {
                get_key_from_db(db, key, None)
            }
        })
}

pub fn debug_get_all_utxos<D>(
    db: D
) -> Result<String>
    where D: DatabaseInterface
{
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| get_all_utxos_as_json_string(db))
}
