use serde_json::json;
use bitcoin_hashes::{
    Hash,
    sha256d,
};
use crate::{
    types::Result,
    traits::DatabaseInterface,
    check_debug_mode::check_debug_mode,
    chains::btc::utxo_manager::utxo_database_utils::{

        get_all_utxo_db_keys,
        delete_last_utxo_key,
        delete_first_utxo_key,
        put_total_utxo_balance_in_db,
        get_utxo_with_tx_id_and_v_out,
    },
};

pub fn clear_all_utxos<D: DatabaseInterface>(db: &D) -> Result<String> {
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .map(|_| get_all_utxo_db_keys(db).to_vec())
        .and_then(|db_keys| db_keys.iter().map(|db_key| db.delete(db_key.to_vec())).collect::<Result<Vec<()>>>())
        .and_then(|_| delete_last_utxo_key(db))
        .and_then(|_| delete_first_utxo_key(db))
        .and_then(|_| put_total_utxo_balance_in_db(db, 0))
        .and_then(|_| db.end_transaction())
        .map(|_| "{clear_all_utxos_succeeded:true}".to_string())
}

pub fn remove_utxo<D: DatabaseInterface>(db: D, tx_id: &str, v_out: u32) -> Result<String> {
    let tx_id_bytes = match hex::decode(tx_id) {
        Ok(bytes) => Ok(bytes),
        Err(_) => Err("Could not decode tx_id hex string!".to_string())
    }?;
    let id = sha256d::Hash::from_slice(&tx_id_bytes)?;
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| get_utxo_with_tx_id_and_v_out(&db, v_out, &id))
        .and_then(|_| db.end_transaction())
        .map(|_| json!({
            "success": "true",
            "v_out_of_removed_utxo": v_out,
            "tx_id_of_removed_utxo": tx_id,
        }).to_string())
}
