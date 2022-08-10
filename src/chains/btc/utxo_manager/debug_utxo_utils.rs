use function_name::named;
use serde_json::json;

use crate::{
    chains::btc::{
        btc_database_utils::BtcDbUtils,
        btc_recipients_and_amounts::BtcRecipientsAndAmounts,
        btc_transaction::create_signed_raw_btc_tx_for_n_input_n_outputs,
        btc_utils::{get_btc_tx_id_from_str, get_hex_tx_from_signed_btc_tx, get_pay_to_pub_key_hash_script},
        extract_utxos_from_p2pkh_txs::extract_utxos_from_p2pkh_txs,
        utxo_manager::{
            utxo_database_utils::{
                delete_first_utxo_key,
                delete_last_utxo_key,
                get_all_utxo_db_keys,
                get_total_number_of_utxos_from_db,
                get_utxo_with_tx_id_and_v_out,
                get_x_utxos,
                save_new_utxo_and_value,
                save_utxos_to_db,
                set_utxo_balance_to_zero,
            },
            utxo_types::BtcUtxosAndValues,
        },
    },
    constants::SUCCESS_JSON,
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    traits::DatabaseInterface,
    types::Result,
};

/// # Debug Clear All UTXOS
///
/// This function will remove ALL UTXOS from the core's encrypted database
///
/// ### BEWARE:
/// Use with extreme caution, and only if you know exactly what you are doing and why.
#[named]
pub fn debug_clear_all_utxos<D: DatabaseInterface>(db: &D, core_type: &CoreType, signature: &str) -> Result<String> {
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .map(|_| get_all_utxo_db_keys(db).to_vec())
        .and_then(|db_keys| {
            db_keys
                .iter()
                .map(|db_key| db.delete(db_key.to_vec()))
                .collect::<Result<Vec<()>>>()
        })
        .and_then(|_| delete_last_utxo_key(db))
        .and_then(|_| delete_first_utxo_key(db))
        .and_then(|_| set_utxo_balance_to_zero(db))
        .and_then(|_| db.end_transaction())
        .map(|_| SUCCESS_JSON.to_string())
}

/// # Debug Remove UTXO
///
/// Pluck a UTXO from the UTXO set and discard it, locating it via its transaction ID and v-out values.
///
/// ### BEWARE:
/// Use ONLY if you know exactly what you're doing and why!
#[named]
pub fn debug_remove_utxo<D: DatabaseInterface>(
    db: &D,
    tx_id: &str,
    v_out: u32,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), tx_id, &v_out, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| get_btc_tx_id_from_str(tx_id))
        .and_then(|id| get_utxo_with_tx_id_and_v_out(db, v_out, &id))
        .and_then(|_| db.end_transaction())
        .map(|_| json!({ "v_out_of_removed_utxo": v_out, "tx_id_of_removed_utxo": tx_id }).to_string())
}

/// # Debug Consolidate Utxos
///
/// This function removes X number of UTXOs from the database then crafts them into a single
/// transcation to itself before returning the serialized output ready for broadcasting, thus
/// consolidating those X UTXOs into a single one.
///
/// ### BEWARE:
/// This function spends UTXOs and outputs a signed transaction. If the outputted transaction is NOT
/// broadcast, the consolidated  output saved in the DB will NOT be spendable, leaving the enclave
/// bricked. Use ONLY if you know exactly what you're doing and why!
#[named]
pub fn debug_consolidate_utxos<D: DatabaseInterface>(
    db: &D,
    fee: u64,
    num_utxos: usize,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    if num_utxos < 1 {
        return Err("Cannot consolidate 0 UTXOs!".into());
    };
    let btc_db_utils = BtcDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), &fee, &num_utxos, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| get_x_utxos(db, num_utxos))
        .and_then(|utxos| {
            let btc_address = btc_db_utils.get_btc_address_from_db()?;
            let target_script = get_pay_to_pub_key_hash_script(&btc_address)?;
            let btc_tx = create_signed_raw_btc_tx_for_n_input_n_outputs(
                fee,
                BtcRecipientsAndAmounts::default(),
                &btc_address,
                &btc_db_utils.get_btc_private_key_from_db()?,
                utxos,
            )?;
            let change_utxos = extract_utxos_from_p2pkh_txs(&target_script, &[btc_tx.clone()]);
            save_utxos_to_db(db, &change_utxos)?;
            Ok(btc_tx)
        })
        .and_then(|btc_tx| {
            let output = json!({
                "fee": fee,
                "num_utxos_spent": num_utxos,
                "btc_tx_hash": btc_tx.txid().to_string(),
                "btc_tx_hex": get_hex_tx_from_signed_btc_tx(&btc_tx),
                "num_utxos_remaining": get_total_number_of_utxos_from_db(db),
            })
            .to_string();
            db.end_transaction()?;
            Ok(output)
        })
}

/// # Debug Get Child-Pays-For-Parent BTC Transaction
///
/// This function attempts to find the UTXO via the passed in transaction hash and vOut values, and
/// upon success creates a transaction spending that UTXO, sending it entirely to itself minus the
/// passed in fee.
///
/// ### BEWARE:
/// This function spends UTXOs and outputs the signed transactions. If the output trnsaction is NOT
/// broadcast, the change output saved in the DB will NOT be spendable, leaving the enclave
/// bricked. Use ONLY if you know exactly what you're doing and why!
#[named]
pub fn debug_get_child_pays_for_parent_btc_tx<D: DatabaseInterface>(
    db: &D,
    fee: u64,
    tx_id: &str,
    v_out: u32,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    let btc_db_utils = BtcDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), &fee, tx_id, &v_out, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| get_btc_tx_id_from_str(tx_id))
        .and_then(|id| get_utxo_with_tx_id_and_v_out(db, v_out, &id))
        .and_then(|utxo| {
            const MAX_FEE_MULTIPLE: u64 = 10;
            let fee_from_db = btc_db_utils.get_btc_fee_from_db()?;
            let btc_address = btc_db_utils.get_btc_address_from_db()?;
            let target_script = get_pay_to_pub_key_hash_script(&btc_address)?;
            if fee > fee_from_db * MAX_FEE_MULTIPLE {
                return Err("Passed in fee is > 10x the fee saved in the db!".into());
            };
            let btc_tx = create_signed_raw_btc_tx_for_n_input_n_outputs(
                fee,
                BtcRecipientsAndAmounts::default(),
                &btc_address,
                &btc_db_utils.get_btc_private_key_from_db()?,
                BtcUtxosAndValues::new(vec![utxo]),
            )?;
            let change_utxos = extract_utxos_from_p2pkh_txs(&target_script, &[btc_tx.clone()]);
            save_utxos_to_db(db, &change_utxos)?;
            db.end_transaction()?;
            Ok(btc_tx)
        })
        .map(|btc_tx| {
            json!({
                "fee": fee,
                "v_out_of_spent_utxo": v_out,
                "tx_id_of_spent_utxo": tx_id,
                "btc_tx_hash": btc_tx.txid().to_string(),
                "btc_tx_hex": get_hex_tx_from_signed_btc_tx(&btc_tx),
            })
            .to_string()
        })
}

/// # Debug Add Multiple Utxos
///
/// Add multiple UTXOs to the databsae. This function first checks if that UTXO already exists in
/// the encrypted database, skipping it if so.
///
/// ### NOTE:
///
/// This function takes as it's argument and valid JSON string in the format that the
/// `debug_get_all_utxos` returns. In this way, it's useful for migrating a UTXO set from one core
/// to another.
///
/// ### BEWARE:
/// Use ONLY if you know exactly what you're doing and why!
#[named]
pub fn debug_add_multiple_utxos<D: DatabaseInterface>(
    db: &D,
    json_str: &str,
    core_type: &CoreType,
    signature: &str,
) -> Result<String> {
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| get_debug_command_hash!(function_name!(), json_str, core_type)())
        .and_then(|hash| validate_debug_command_signature(db, core_type, signature, &hash))
        .and_then(|_| BtcUtxosAndValues::from_str(json_str))
        .and_then(|utxos| {
            utxos
                .iter()
                .map(|utxo| save_new_utxo_and_value(db, utxo))
                .collect::<Result<Vec<()>>>()
        })
        .and_then(|_| {
            db.end_transaction()?;
            Ok(SUCCESS_JSON.to_string())
        })
}

#[cfg(all(features = "debug", test))]
mod tests {
    use super::*;
    use crate::{
        chains::btc::{
            btc_test_utils::get_sample_utxo_and_values,
            utxo_manager::{
                utxo_database_utils::{get_total_utxo_balance_from_db, save_utxos_to_db},
                utxo_utils::get_all_utxos_as_json_string,
            },
        },
        test_utils::{get_test_database, DUMMY_DEBUG_COMMAND_HASH, DUMMY_SIGNATURE},
    };

    #[test]
    fn should_clear_all_utxos() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        let expected_balance = utxos.sum();
        save_utxos_to_db(&db, &utxos).unwrap();
        let mut balance = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(expected_balance, balance);
        debug_clear_all_utxos(&db, &CoreType::default(), DUMMY_SIGNATURE, DUMMY_DEBUG_COMMAND_HASH).unwrap();
        balance = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(0, balance);
    }

    #[test]
    fn should_insert_multiple_utxos() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        let expected_balance = utxos.sum();
        save_utxos_to_db(&db, &utxos).unwrap();
        let mut balance = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(expected_balance, balance);
        let json = get_all_utxos_as_json_string(&db).unwrap();
        let core_type = CoreType::default();
        debug_clear_all_utxos(&db, &core_type, DUMMY_SIGNATURE, DUMMY_DEBUG_COMMAND_HASH).unwrap();
        balance = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(0, balance);
        debug_add_multiple_utxos(&db, &json, &core_type, DUMMY_SIGNATURE, DUMMY_DEBUG_COMMAND_HASH).unwrap();
        balance = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(expected_balance, balance);
    }
}
