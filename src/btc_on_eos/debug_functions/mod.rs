#![allow(dead_code)] // TODO rm!
#![allow(unused_imports)] // TODO rm!
use crate::{
    types::Result,
    traits::DatabaseInterface,
    check_debug_mode::check_debug_mode,
    btc_on_eos::{
        utxo_manager::utxo_database_utils::{
            get_utxo_from_db,
            get_all_utxo_db_keys,
        },
        check_core_is_initialized::{
            check_core_is_initialized,
            check_core_is_initialized_and_return_eos_state,
            check_core_is_initialized_and_return_btc_state,
        },
        eos::{
            eos_database_utils::put_eos_schedule_in_db,
            eos_constants::EOS_PRIVATE_KEY_DB_KEY as EOS_KEY,
            parse_submission_material::parse_producer_schedule_from_json_string,
        },
        btc::{
            btc_state::BtcState,
            btc_types::BtcUtxoAndValue,
            save_utxos_to_db::maybe_save_utxos_to_db,
            filter_utxos::maybe_filter_utxos_in_state,
            btc_constants::BTC_PRIVATE_KEY_DB_KEY as BTC_KEY,
            validate_btc_merkle_root::validate_btc_merkle_root,
            filter_minting_params::maybe_filter_minting_params_in_state,
            validate_btc_block_header::validate_btc_block_header_in_state,
            filter_p2sh_deposit_txs::filter_p2sh_deposit_txs_and_add_to_state,
            btc_database_utils::{
                end_btc_db_transaction,
                start_btc_db_transaction,
            },
            get_deposit_info_hash_map::{
                get_deposit_info_hash_map_and_put_in_state,
            },
            validate_btc_proof_of_work::{
                validate_proof_of_work_of_btc_block_in_state,
            },
            extract_utxos_from_p2sh_txs::{
                maybe_extract_utxos_from_p2sh_txs_and_put_in_state
            },
            parse_minting_params_from_p2sh_deposits::{
                parse_minting_params_from_p2sh_deposits_and_add_to_state,
            },
        },
    },
};

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
        .and_then(|_| parse_producer_schedule_from_json_string(&schedule_json))
        .and_then(|schedule| put_eos_schedule_in_db(&db, &schedule))
        .and_then(|_| db.end_transaction())
        .map(|_| "{debug_adding_eos_schedule_succeeded:true}".to_string())
}

/*
pub fn debug_reprocess_btc_block<D>(
    db: D,
    btc_block_json: String,
) -> Result<String>
    where D: DatabaseInterface
{
    parse_btc_block_and_id_and_put_in_state(
        btc_block_json,
        BtcState::init(db),
    )
        .and_then(check_core_is_initialized_and_return_btc_state)
        .and_then(start_btc_db_transaction)
        .and_then(validate_btc_block_header_in_state)
        .and_then(validate_proof_of_work_of_btc_block_in_state)
        .and_then(validate_btc_merkle_root)
        .and_then(get_deposit_info_hash_map_and_put_in_state)
        .and_then(filter_op_return_deposit_txs_and_add_to_state)
        .and_then(filter_p2sh_deposit_txs_and_add_to_state)
        .and_then(parse_minting_params_from_op_return_deposits_and_add_to_state)
        .and_then(parse_minting_params_from_p2sh_deposits_and_add_to_state)
        .and_then(maybe_extract_utxos_from_op_return_txs_and_put_in_state)
        .and_then(maybe_extract_utxos_from_p2sh_txs_and_put_in_state)
        .and_then(maybe_filter_utxos_in_state)
        .and_then(maybe_save_utxos_to_db)
        .and_then(maybe_filter_minting_params_in_state)
        .and_then(|state| {
            get_eth_signed_txs(
                &get_signing_params_from_db(&state.db)?,
                &state.minting_params,
            )
                .and_then(|signed_txs| state.add_eth_signed_txs(signed_txs))
        })
        .and_then(maybe_increment_eth_nonce_in_db)
        .and_then(|state| {
            let signatures = serde_json::to_string(
                &match &state.eth_signed_txs {
                    None => Ok(vec![]),
                    Some(txs) =>
                        get_eth_signed_tx_info_from_eth_txs(
                            txs,
                            &state.minting_params,
                            get_eth_account_nonce_from_db(&state.db)?,
                        )
                }?
            )?;
            info!("✔ BTC signatures: {}", signatures);
            state.add_output_json_string(signatures)
        })
        .and_then(end_btc_db_transaction)
        .map(|state|
            match state.output_json_string {
                None => format!("✘ No signatures signed ∴ no output!"),
                Some(output) => output
            }
        )
}
*/

pub fn debug_set_key_in_db_to_value<D>(
    db: D,
    key: String,
    value: String
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Setting key: {} in DB to value: {}", key, value);
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_| db.start_transaction())
        .and_then(|_| db.put(hex::decode(key)?, hex::decode(value)?, None))
        .and_then(|_| db.end_transaction())
        .map(|_| "{putting_value_in_database_suceeded:true}".to_string())
}

pub fn debug_get_key_from_db<D>(
    db: D,
    key: String
) -> Result<String>
    where D: DatabaseInterface
{
    info!("✔ Maybe getting key: {} from DB...", key);
    let key_bytes = hex::decode(&key)?;
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_|
            match key_bytes == EOS_KEY || key_bytes == BTC_KEY {
                false => db.get(hex::decode(key.clone())?, None),
                true => db.get(hex::decode(key.clone())?, Some(255)),
            }
        )
        .map(|value|
            format!(
                "{{key:{},value:{}}}",
                key,
                hex::encode(value),
            )
        )
}

pub fn debug_get_all_utxos<D>(
    db: D
) -> Result<String>
    where D: DatabaseInterface
{
    #[derive(Serialize, Deserialize)]
    struct UtxoDetails {
        pub db_key: String,
        pub db_value: String,
        pub utxo_and_value: BtcUtxoAndValue,
    }
    check_debug_mode()
        .and_then(|_| check_core_is_initialized(&db))
        .and_then(|_|
            Ok(
                serde_json::to_string(
                    &get_all_utxo_db_keys(&db)
                        .iter()
                        .map(|db_key| {
                            Ok(
                                UtxoDetails {
                                    db_key:
                                        hex::encode(db_key.to_vec()),
                                    utxo_and_value:
                                        get_utxo_from_db(&db, &db_key.to_vec())?,
                                    db_value:
                                        hex::encode(
                                            db.get(db_key.to_vec(), None)?
                                        ),
                                }
                            )
                        })
                        .map(|utxo_details: Result<UtxoDetails>| utxo_details)
                        .flatten()
                        .collect::<Vec<UtxoDetails>>()
                )?
            )
        )
}
