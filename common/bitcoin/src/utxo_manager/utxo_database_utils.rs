use common::{
    errors::AppError,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::{convert_bytes_to_u64, convert_u64_to_bytes},
};

use crate::{
    bitcoin_crate_alias::{
        hashes::{sha256d, Hash},
        Txid,
    },
    utxo_manager::{
        utxo_constants::{UTXO_BALANCE, UTXO_FIRST, UTXO_LAST, UTXO_NONCE},
        utxo_types::{BtcUtxoAndValue, BtcUtxosAndValues},
        utxo_utils::{deserialize_utxo_and_value, get_utxo_and_value_db_key, serialize_btc_utxo_and_value},
    },
};

// FIXME Make a struct of this ala BtdDbutils

fn put_u64_in_db<D: DatabaseInterface>(db: &D, key: &[Byte], u_64: u64) -> Result<()> {
    debug!("✔ Putting `u64` of {} in db...", u_64);
    db.put(key.to_vec(), u_64.to_le_bytes().to_vec(), None)
}

fn get_u64_from_db<D: DatabaseInterface>(db: &D, key: &[Byte]) -> Result<u64> {
    debug!("✔ Getting `u64` from db...");
    db.get(key.to_vec(), None)
        .and_then(|ref bytes| convert_bytes_to_u64(bytes))
}

pub fn get_x_utxos<D: DatabaseInterface>(db: &D, num_utxos_to_get: usize) -> Result<BtcUtxosAndValues> {
    let total_num_utxos = get_total_number_of_utxos_from_db(db);
    if total_num_utxos < num_utxos_to_get {
        return Err(format!(
            "Can't get {} UTXOS, there're only {} in the db!",
            num_utxos_to_get, total_num_utxos
        )
        .into());
    };
    fn get_utxos_recursively<D: DatabaseInterface>(
        db: &D,
        num_utxos_to_get: usize,
        mut utxos: Vec<BtcUtxoAndValue>,
    ) -> Result<BtcUtxosAndValues> {
        get_first_utxo_and_value(db).and_then(|utxo| {
            utxos.push(utxo);
            if utxos.len() == num_utxos_to_get {
                Ok(BtcUtxosAndValues::new(utxos))
            } else {
                get_utxos_recursively(db, num_utxos_to_get, utxos)
            }
        })
    }
    get_utxos_recursively(db, num_utxos_to_get, vec![])
}

fn remove_utxo_pointer(utxo: &BtcUtxoAndValue) -> BtcUtxoAndValue {
    let mut utxo_with_no_pointer = utxo.clone();
    utxo_with_no_pointer.maybe_pointer = None;
    utxo_with_no_pointer
}

fn utxo_exists<D: DatabaseInterface>(db: &D, v_out: u32, tx_id: &Txid) -> bool {
    fn utxo_exists_recursive<D: DatabaseInterface>(
        db: &D,
        tx_id: &Txid,
        v_out: u32,
        maybe_pointer: Option<Bytes>,
    ) -> Result<BtcUtxoAndValue> {
        let maybe_utxo = match maybe_pointer {
            None => get_utxo_from_db(db, &get_first_utxo_pointer(db)?),
            Some(ref pointer) => get_utxo_from_db(db, pointer),
        };
        match maybe_utxo {
            Err(_) => Err("Could not get UTXO!".into()),
            Ok(utxo) => {
                if &utxo.get_tx_id()? == tx_id && utxo.get_v_out()? == v_out {
                    Ok(utxo)
                } else {
                    match utxo.maybe_pointer {
                        Some(next_pointer) => utxo_exists_recursive(db, tx_id, v_out, Some(next_pointer.to_vec())),
                        None => Err("No more UTXOs to search!".into()),
                    }
                }
            },
        }
    }

    utxo_exists_recursive(db, tx_id, v_out, None).is_ok()
}

pub fn get_utxo_with_tx_id_and_v_out<D: DatabaseInterface>(
    db: &D,
    v_out: u32,
    tx_id: &Txid,
) -> Result<BtcUtxoAndValue> {
    fn find_utxo_recursively<D: DatabaseInterface>(
        db: &D,
        v_out: u32,
        tx_id: &Txid,
        mut utxos: Vec<BtcUtxoAndValue>,
    ) -> Result<(Option<BtcUtxoAndValue>, BtcUtxosAndValues)> {
        match get_first_utxo_and_value(db) {
            Err(_) => Ok((None, BtcUtxosAndValues::new(utxos))),
            Ok(utxo) => {
                if utxo.get_v_out()? == v_out && &utxo.get_tx_id()? == tx_id {
                    Ok((Some(utxo), BtcUtxosAndValues::new(utxos)))
                } else {
                    utxos.push(remove_utxo_pointer(&utxo));
                    find_utxo_recursively(db, v_out, tx_id, utxos)
                }
            },
        }
    }
    find_utxo_recursively(db, v_out, tx_id, vec![]).and_then(|(maybe_utxo, utxos_to_save_in_db)| {
        save_utxos_to_db(db, &utxos_to_save_in_db)?;
        maybe_utxo
            .ok_or_else(|| AppError::Custom(format!("Could not find UTXO with v_out: {} & tx_id: {}", v_out, tx_id)))
    })
}

pub fn save_utxos_to_db<D: DatabaseInterface>(db: &D, utxos_and_values: &BtcUtxosAndValues) -> Result<()> {
    debug!("✔ Saving {} `utxo_and_value`s...", utxos_and_values.len());
    utxos_and_values
        .0
        .iter()
        .try_for_each(|utxo_and_value| save_new_utxo_and_value(db, utxo_and_value))
}

pub fn get_all_utxo_db_keys<D: DatabaseInterface>(db: &D) -> Vec<Bytes> {
    fn get_utxo_pointers_recursively<D: DatabaseInterface>(db: &D, mut pointers: Vec<Bytes>) -> Vec<Bytes> {
        match maybe_get_next_utxo_pointer_from_utxo_pointer(db, &pointers[pointers.len() - 1]) {
            None => pointers,
            Some(next_pointer) => {
                pointers.push(next_pointer);
                get_utxo_pointers_recursively(db, pointers)
            },
        }
    }
    match get_first_utxo_pointer(db) {
        Ok(first_pointer) => get_utxo_pointers_recursively(db, vec![first_pointer]),
        _ => vec![],
    }
}

fn maybe_get_next_utxo_pointer_from_utxo_pointer<D: DatabaseInterface>(db: &D, utxo_pointer: &[Byte]) -> Option<Bytes> {
    match maybe_get_utxo_from_db(db, utxo_pointer) {
        None => None,
        Some(utxo) => utxo.maybe_pointer.map(|pointer| pointer.to_vec()),
    }
}

pub fn get_first_utxo_and_value<D: DatabaseInterface>(db: &D) -> Result<BtcUtxoAndValue> {
    get_first_utxo_pointer(db)
        .and_then(|pointer| get_utxo_from_db(db, &pointer))
        .and_then(|utxo| match utxo.maybe_pointer {
            None => {
                debug!("✔ No next pointer ∴ must be last UTXO in db!");
                set_utxo_balance_to_zero(db)
                    .and_then(|_| delete_first_utxo(db))
                    .and_then(|_| delete_last_utxo_key(db))
                    .and_then(|_| delete_first_utxo_key(db))
                    .map(|_| utxo)
            },
            Some(pointer) => {
                debug!("✔ UTXO found, updating `UTXO_FIRST` pointer...");
                decrement_total_utxo_balance_in_db(db, utxo.value)
                    .and_then(|_| delete_first_utxo(db))
                    .and_then(|_| set_first_utxo_pointer(db, &pointer))
                    .map(|_| utxo)
            },
        })
}

pub fn save_new_utxo_and_value<D: DatabaseInterface>(db: &D, utxo_and_value: &BtcUtxoAndValue) -> Result<()> {
    // NOTE: We clear any extant pointers since we definitely don't want any when inserting a new UTXO!
    // NOTE: This case could crop up when adding UTXOs via a JSON dumped from the DB for example.
    let mut utxo = utxo_and_value.clone();
    utxo.maybe_pointer = None;
    if utxo_exists(db, utxo.get_v_out()?, &utxo.get_tx_id()?) {
        info!("✘ Not saving UTXO ∵ it's already in the database!");
        Ok(())
    } else {
        let value = utxo.value;
        let hash_vec = get_utxo_and_value_db_key(get_utxo_nonce_from_db(db)? + 1);
        let hash = sha256d::Hash::from_slice(&hash_vec)?;
        debug!("✔ Saving new UTXO in db under hash: {}", hex::encode(hash));
        if get_total_utxo_balance_from_db(db)? == 0 {
            debug!("✔ UTXO balance == 0, ∴ setting `UTXO_FIRST` & `UTXO_LAST` db keys!");
            set_first_utxo_pointer(db, &hash)
                .and_then(|_| increment_utxo_nonce_in_db(db))
                .and_then(|_| set_last_utxo_pointer(db, &hash))
                .and_then(|_| put_total_utxo_balance_in_db(db, value))
                .and_then(|_| put_utxo_in_db(db, &hash_vec, &utxo))
        } else {
            debug!("✔ UTXO balance is > 0 ∴ only setting `UTXO_LAST` db key!");
            update_pointer_in_last_utxo_in_db(db, hash)
                .and_then(|_| increment_utxo_nonce_in_db(db))
                .and_then(|_| set_last_utxo_pointer(db, &hash))
                .and_then(|_| put_utxo_in_db(db, &hash_vec, &utxo))
                .and_then(|_| increment_total_utxo_balance_in_db(db, value))
        }
    }
}

pub fn delete_last_utxo_key<D: DatabaseInterface>(db: &D) -> Result<()> {
    debug!("✔ Deleting `UTXO_LAST` key from db...");
    db.delete(UTXO_LAST.to_vec())
}

pub fn delete_first_utxo_key<D: DatabaseInterface>(db: &D) -> Result<()> {
    debug!("✔ Deleting `UTXO_FIRST` key from db...");
    db.delete(UTXO_FIRST.to_vec())
}

pub fn delete_first_utxo<D: DatabaseInterface>(db: &D) -> Result<()> {
    get_first_utxo_pointer(db).and_then(|pointer| {
        debug!("✔ Deleting UTXO under key: {}", hex::encode(&pointer));
        db.delete(pointer.to_vec())
    })
}

pub fn set_utxo_balance_to_zero<D: DatabaseInterface>(db: &D) -> Result<()> {
    debug!("✔ Setting UTXO balance to zero...");
    put_total_utxo_balance_in_db(db, 0)
}

pub fn increment_total_utxo_balance_in_db<D: DatabaseInterface>(db: &D, amount_to_increment_by: u64) -> Result<()> {
    get_total_utxo_balance_from_db(db).and_then(|balance| {
        debug!("✔ Incrementing UTXO total by: {}", amount_to_increment_by);
        put_total_utxo_balance_in_db(db, balance + amount_to_increment_by)
    })
}

pub fn decrement_total_utxo_balance_in_db<D: DatabaseInterface>(db: &D, amount_to_decrement_by: u64) -> Result<()> {
    get_total_utxo_balance_from_db(db).and_then(|balance| match balance >= amount_to_decrement_by {
        true => {
            debug!("✔ Decrementing UTXO balance by {}", amount_to_decrement_by);
            put_total_utxo_balance_in_db(db, balance - amount_to_decrement_by)
        },
        false => Err("✘ Not decrementing UTXO total value ∵ it'll underflow!".into()),
    })
}

pub fn put_total_utxo_balance_in_db<D: DatabaseInterface>(db: &D, balance: u64) -> Result<()> {
    debug!("✔ Setting total UTXO balance to: {}", balance);
    put_u64_in_db(db, &UTXO_BALANCE.to_vec(), balance)
}

pub fn get_total_utxo_balance_from_db<D: DatabaseInterface>(db: &D) -> Result<u64> {
    debug!("✔ Getting total UTXO balance from db...");
    get_u64_from_db(db, &UTXO_BALANCE.to_vec()).map_err(|_| "Error getting UTXO balance from db!".into())
}

pub fn update_pointer_in_last_utxo_in_db<D: DatabaseInterface>(db: &D, new_pointer: sha256d::Hash) -> Result<()> {
    debug!("✔ Updating `UTXO_LAST` pointer in db to {}", new_pointer);
    get_last_utxo_pointer(db)
        .and_then(|pointer_to_utxo| update_pointer_in_utxo_in_db(db, &pointer_to_utxo, new_pointer))
}

pub fn update_pointer_in_utxo_in_db<D: DatabaseInterface>(
    db: &D,
    db_key: &[Byte],
    new_pointer: sha256d::Hash,
) -> Result<()> {
    debug!(
        "✔ Updating UTXO pointer in db under key: {} to: {}",
        hex::encode(db_key),
        new_pointer
    );
    get_utxo_from_db(db, db_key)
        .map(|utxo| utxo.update_pointer(new_pointer))
        .and_then(|utxo| put_utxo_in_db(db, db_key, &utxo))
}

pub fn maybe_get_utxo_from_db<D: DatabaseInterface>(db: &D, db_key: &[Byte]) -> Option<BtcUtxoAndValue> {
    debug!("✔ Maybe getting UTXO in db under key: {}", hex::encode(db_key));
    match db.get(db_key.to_vec(), None) {
        Err(_) => {
            debug!("✘ No UTXO exists in the database @ that key!");
            None
        },
        Ok(bytes) => match deserialize_utxo_and_value(&bytes) {
            Ok(utxo_and_value) => Some(utxo_and_value),
            Err(_) => {
                debug!("✘ Error deserializing UTXO & value!");
                None
            },
        },
    }
}

pub fn get_utxo_from_db<D: DatabaseInterface>(db: &D, db_key: &[Byte]) -> Result<BtcUtxoAndValue> {
    debug!("✔ Getting UTXO in db under key: {}", hex::encode(db_key));
    db.get(db_key.to_vec(), None)
        .and_then(|bytes| deserialize_utxo_and_value(&bytes))
}

pub fn put_utxo_in_db<D: DatabaseInterface>(db: &D, key: &[Byte], utxo: &BtcUtxoAndValue) -> Result<()> {
    debug!("✔ Putting UTXO in db under key: {}", sha256d::Hash::from_slice(key)?);
    db.put(key.to_vec(), serialize_btc_utxo_and_value(utxo)?, None)
}

pub fn set_last_utxo_pointer<D: DatabaseInterface>(db: &D, hash: &sha256d::Hash) -> Result<()> {
    debug!("✔ Setting `UTXO_LAST` pointer to: {}", hash);
    db.put(UTXO_LAST.to_vec(), hash.to_vec(), None)
}

pub fn get_last_utxo_pointer<D: DatabaseInterface>(db: &D) -> Result<Bytes> {
    debug!("✔ Getting `UTXO_LAST` pointer...");
    db.get(UTXO_LAST.to_vec(), None)
}

pub fn set_first_utxo_pointer<D: DatabaseInterface>(db: &D, hash: &sha256d::Hash) -> Result<()> {
    debug!("✔ Setting `UTXO_FIRST` pointer to: {}", hex::encode(hash));
    db.put(UTXO_FIRST.to_vec(), hash.to_vec(), None)
}

pub fn get_first_utxo_pointer<D: DatabaseInterface>(db: &D) -> Result<Bytes> {
    debug!("✔ Getting `UTXO_FIRST` pointer...");
    db.get(UTXO_FIRST.to_vec(), None)
        .map_err(|_| "✘ No UTXOs in the database! Have you bricked this core?".into())
}

pub fn get_utxo_nonce_from_db<D: DatabaseInterface>(db: &D) -> Result<u64> {
    debug!("✔ Getting UTXO nonce from db...");
    match db.get(UTXO_NONCE.to_vec(), None) {
        Err(_) => {
            debug!("✘ Error getting UTXO nonce!");
            Ok(0)
        },
        Ok(bytes) => {
            debug!("✔ Converting bytes to usize for UTXO nonce...");
            convert_bytes_to_u64(&bytes)
        },
    }
}

pub fn get_total_number_of_utxos_from_db<D: DatabaseInterface>(db: &D) -> usize {
    debug!("✔ Getting total number of UTXOs from db...");
    get_all_utxo_db_keys(db).len()
}

pub fn put_utxo_nonce_in_db<D: DatabaseInterface>(db: &D, utxo_nonce: u64) -> Result<()> {
    debug!("✔ Setting UTXO nonce to: {}", utxo_nonce);
    db.put(UTXO_NONCE.to_vec(), convert_u64_to_bytes(utxo_nonce), None)
}

pub fn increment_utxo_nonce_in_db<D: DatabaseInterface>(db: &D) -> Result<()> {
    debug!("✔ Incrementing UTXO nonce in db by 1...");
    get_utxo_nonce_from_db(db).and_then(|num| put_utxo_nonce_in_db(db, num + 1))
}

#[cfg(all(test, not(feature = "ltc")))]
mod tests {
    use common::{errors::AppError, test_utils::get_test_database};

    use super::*;
    use crate::{
        btc_database_utils::BtcDbUtils,
        test_utils::{get_sample_p2pkh_utxo_and_value, get_sample_utxo_and_values},
    };

    fn remove_utxo_pointers(utxos: &BtcUtxosAndValues) -> BtcUtxosAndValues {
        BtcUtxosAndValues::new(utxos.iter().map(remove_utxo_pointer).collect())
    }

    fn get_all_utxos_without_removing_from_db<D: DatabaseInterface>(db: &D) -> Result<BtcUtxosAndValues> {
        Ok(BtcUtxosAndValues::new(
            get_all_utxo_db_keys(db)
                .iter()
                .map(|key| get_utxo_from_db(db, key))
                .collect::<Result<Vec<BtcUtxoAndValue>>>()
                .unwrap(),
        ))
    }

    #[test]
    fn should_be_zero_utxos_when_non_in_db() {
        let db = get_test_database();
        let result = get_utxo_nonce_from_db(&db);
        assert!(result.is_ok());
    }

    #[test]
    fn should_put_num_of_utxos_in_db() {
        let db = get_test_database();
        let num = 1337;
        put_utxo_nonce_in_db(&db, num).unwrap();
        let result = get_utxo_nonce_from_db(&db).unwrap();
        assert_eq!(result, num);
    }

    #[test]
    fn should_increment_num_of_utxos_in_db() {
        let db = get_test_database();
        let num = 1336;
        put_utxo_nonce_in_db(&db, num).unwrap();
        increment_utxo_nonce_in_db(&db).unwrap();
        let result = get_utxo_nonce_from_db(&db).unwrap();
        assert_eq!(result, num + 1);
    }

    #[test]
    fn should_set_and_get_last_utxo_pointer() {
        let db = get_test_database();
        let pointer = "pBTC last".to_string();
        let pointer_hash = sha256d::Hash::hash(pointer.as_bytes());
        set_last_utxo_pointer(&db, &pointer_hash).unwrap();
        let result = get_last_utxo_pointer(&db).unwrap();
        assert_eq!(result, pointer_hash.to_vec());
    }

    #[test]
    fn should_set_and_get_fist_utxo_pointer() {
        let db = get_test_database();
        let pointer = "pBTC first".to_string();
        let pointer_hash = sha256d::Hash::hash(pointer.as_bytes());
        set_first_utxo_pointer(&db, &pointer_hash).unwrap();
        let result = get_first_utxo_pointer(&db).unwrap();
        assert_eq!(result, pointer_hash.to_vec());
    }

    #[test]
    fn should_put_and_get_utxo_in_db() {
        let db = get_test_database();
        let utxo = get_sample_p2pkh_utxo_and_value();
        let key = get_utxo_and_value_db_key(1);
        put_utxo_in_db(&db, &key, &utxo).unwrap();
        let result = get_utxo_from_db(&db, &key).unwrap();
        assert_eq!(result, utxo);
    }

    #[test]
    fn should_update_pointer_in_utxo_in_db() {
        let db = get_test_database();
        let utxo = get_sample_p2pkh_utxo_and_value();
        let key = get_utxo_and_value_db_key(1);
        let pointer = sha256d::Hash::hash(&[6u8, 6u8, 6u8]);
        assert_eq!(utxo.maybe_pointer, None);
        put_utxo_in_db(&db, &key, &utxo).unwrap();
        update_pointer_in_utxo_in_db(&db, &key, pointer).unwrap();
        let result = get_utxo_from_db(&db, &key).unwrap();
        assert_eq!(result.maybe_pointer, Some(pointer));
    }

    #[test]
    fn should_set_and_get_total_utxo_balance_from_db() {
        let num = 1337;
        let db = get_test_database();
        put_total_utxo_balance_in_db(&db, num).unwrap();
        let result = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(result, num);
    }

    #[test]
    fn should_increment_total_utxo_balance_in_db() {
        let db = get_test_database();
        let num = 666;
        let expected_total = 1337;
        let amount_to_increment = 671;
        put_total_utxo_balance_in_db(&db, num).unwrap();
        increment_total_utxo_balance_in_db(&db, amount_to_increment).unwrap();
        let result = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(result, expected_total);
    }

    #[test]
    fn should_decrement_total_utxo_balance_in_db() {
        let db = get_test_database();
        let num = 1337;
        let expected_total = 666;
        let amount_to_decrement_by = 671;
        put_total_utxo_balance_in_db(&db, num).unwrap();
        decrement_total_utxo_balance_in_db(&db, amount_to_decrement_by).unwrap();
        let result = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(result, expected_total);
    }

    #[test]
    fn should_err_when_decrementing_with_underflow() {
        let db = get_test_database();
        let num = 1337;
        let amount_to_decrement_by = num + 1;
        assert!(amount_to_decrement_by > num);
        let expected_error = "✘ Not decrementing UTXO total value ∵ it'll underflow!".to_string();
        put_total_utxo_balance_in_db(&db, num).unwrap();
        match decrement_total_utxo_balance_in_db(&db, amount_to_decrement_by) {
            Ok(_) => panic!("Decrementing balance of utxos should error!"),
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error on decrement UTXO balance: {}", e),
        };
    }

    #[test]
    fn should_zero_utxo_balance() {
        let db = get_test_database();
        let balance = 1;
        put_total_utxo_balance_in_db(&db, balance).unwrap();
        let balance_before = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(balance_before, balance);
        set_utxo_balance_to_zero(&db).unwrap();
        let balance_after = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(balance_after, 0);
    }

    #[test]
    fn should_delete_first_key() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let hash = sha256d::Hash::hash(&[1u8]);
        set_first_utxo_pointer(&db, &hash).unwrap();
        delete_first_utxo_key(&db).unwrap();
        let result = db_utils.key_exists_in_db(&UTXO_FIRST.to_vec(), None);
        assert!(!result);
    }

    #[test]
    fn should_delete_last_key() {
        let db = get_test_database();
        let db_utils = BtcDbUtils::new(&db);
        let hash = sha256d::Hash::hash(&[1u8]);
        set_last_utxo_pointer(&db, &hash).unwrap();
        delete_last_utxo_key(&db).unwrap();
        let result = db_utils.key_exists_in_db(&UTXO_LAST.to_vec(), None);
        assert!(!result);
    }

    #[test]
    fn should_save_gt_one_utxo() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        let utxo1 = utxos[0].clone();
        let hash1 = get_utxo_and_value_db_key(1);
        let mut utxo2 = utxos[1].clone();
        let hash2 = get_utxo_and_value_db_key(2);
        let hash = sha256d::Hash::hash(b"a hash");
        utxo2.maybe_pointer = Some(hash);
        assert!(utxo1 != utxo2);
        save_new_utxo_and_value(&db, &utxo1).unwrap();
        let utxo_nonce = get_utxo_nonce_from_db(&db).unwrap();
        assert_eq!(utxo_nonce, 1);
        let mut first_pointer = get_first_utxo_pointer(&db).unwrap();
        assert_eq!(first_pointer, hash1);
        let mut last_pointer = get_last_utxo_pointer(&db).unwrap();
        assert_eq!(last_pointer, hash1);
        save_new_utxo_and_value(&db, &utxo2).unwrap();
        first_pointer = get_first_utxo_pointer(&db).unwrap();
        assert_eq!(first_pointer, hash1);
        last_pointer = get_last_utxo_pointer(&db).unwrap();
        assert_eq!(last_pointer, hash2);
        let result = get_utxo_from_db(&db, &hash1).unwrap();
        let expected_pointer = Some(sha256d::Hash::from_slice(&hash2).unwrap());
        assert_eq!(result.value, utxo1.value);
        assert_eq!(result.maybe_pointer, expected_pointer);
        assert_eq!(result.serialized_utxo, utxo1.serialized_utxo);
    }

    #[test]
    fn should_remove_1_utxo_correctly_when_gt_1_exist() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        let utxo1 = utxos[0].clone();
        let hash1 = get_utxo_and_value_db_key(1);
        let mut utxo2 = utxos[1].clone();
        let hash2 = get_utxo_and_value_db_key(2);
        let hash = sha256d::Hash::hash(b"a hash");
        utxo2.maybe_pointer = Some(hash);
        let mut expected_utxo1 = utxo1.clone();
        expected_utxo1.maybe_pointer = Some(sha256d::Hash::from_slice(&hash2).unwrap());
        assert!(utxo1 != utxo2);
        save_new_utxo_and_value(&db, &utxo1).unwrap();
        save_new_utxo_and_value(&db, &utxo2).unwrap();
        let nonce = get_utxo_nonce_from_db(&db).unwrap();
        assert_eq!(nonce, 2);
        let mut first_pointer = get_first_utxo_pointer(&db).unwrap();
        assert_eq!(first_pointer, hash1);
        let mut last_pointer = get_last_utxo_pointer(&db).unwrap();
        assert_eq!(last_pointer, hash2);
        let utxo = get_first_utxo_and_value(&db).unwrap();
        assert_eq!(utxo, expected_utxo1);
        first_pointer = get_first_utxo_pointer(&db).unwrap();
        assert_eq!(first_pointer, hash2);
        last_pointer = get_last_utxo_pointer(&db).unwrap();
        assert_eq!(last_pointer, hash2);
    }

    #[test]
    fn should_remove_last_utxo_correctly() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxo1 = get_sample_p2pkh_utxo_and_value();
        save_new_utxo_and_value(&db, &utxo1).unwrap();
        let first_pointer_before = get_first_utxo_pointer(&db).unwrap();
        let last_pointer_before = get_last_utxo_pointer(&db).unwrap();
        let utxo_total_before = get_total_utxo_balance_from_db(&db).unwrap();
        get_first_utxo_and_value(&db).unwrap();
        let first_pointer_after = get_first_utxo_pointer(&db);
        let last_pointer_after = get_last_utxo_pointer(&db);
        let utxo_total_after = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(utxo_total_after, 0);
        assert!(last_pointer_after.is_err());
        assert!(first_pointer_after.is_err());
        assert_eq!(utxo_total_before, utxo1.value);
        assert!(utxo_total_after < utxo_total_before);
        assert_eq!(first_pointer_before, last_pointer_before);
    }

    #[test]
    fn should_delete_first_utxo_in_db() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let db_utils = BtcDbUtils::new(&db);
        let utxos = get_sample_utxo_and_values();
        let first_utxo_db_key = get_utxo_and_value_db_key(1);
        save_utxos_to_db(&db, &utxos).unwrap();
        assert!(db_utils.key_exists_in_db(&first_utxo_db_key, None));
        delete_first_utxo(&db).unwrap();
        assert!(!db_utils.key_exists_in_db(&first_utxo_db_key, None));
    }

    #[test]
    fn removed_utxos_should_no_longer_be_in_db() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let db_utils = BtcDbUtils::new(&db);
        let utxos = get_sample_utxo_and_values();
        save_utxos_to_db(&db, &utxos).unwrap();
        utxos
            .0
            .iter()
            .enumerate()
            .for_each(|(i, _)| assert!(db_utils.key_exists_in_db(&get_utxo_and_value_db_key((i + 1) as u64), None)));
        assert_eq!(get_utxo_nonce_from_db(&db).unwrap(), utxos.len() as u64);
        assert_eq!(get_first_utxo_pointer(&db).unwrap(), get_utxo_and_value_db_key(1));
        get_first_utxo_and_value(&db).unwrap();
        assert_eq!(get_first_utxo_pointer(&db).unwrap(), get_utxo_and_value_db_key(2));
        assert!(!db_utils.key_exists_in_db(&get_utxo_and_value_db_key(1), None));
    }

    #[test]
    fn should_get_all_utxos_from_db_without_removing_them() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        save_utxos_to_db(&db, &utxos).unwrap();
        let utxos_from_db = get_all_utxos_without_removing_from_db(&db).unwrap();
        let result = remove_utxo_pointers(&utxos_from_db);
        assert_eq!(result, utxos);
    }

    fn should_get_utxo_with_tx_id_and_v_out_correctly(utxos: BtcUtxosAndValues, utxo_to_find_index: usize) {
        assert!(utxo_to_find_index <= utxos.len());
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxo_to_find = utxos[utxo_to_find_index].clone();
        let v_out = utxo_to_find.get_v_out().unwrap();
        let tx_id = utxo_to_find.get_tx_id().unwrap();
        let mut expected_utxos_from_db_after = utxos.clone();
        expected_utxos_from_db_after.remove(utxo_to_find_index);
        save_utxos_to_db(&db, &utxos).unwrap();
        let utxos_from_db_before = get_all_utxos_without_removing_from_db(&db).unwrap();
        assert_eq!(utxos_from_db_before.len(), utxos.len());
        let result = get_utxo_with_tx_id_and_v_out(&db, v_out, &tx_id).unwrap();
        assert_eq!(remove_utxo_pointer(&result), utxo_to_find);
        let utxos_from_db_after = get_all_utxos_without_removing_from_db(&db).unwrap();
        assert_eq!(utxos_from_db_after.len(), utxos.len() - 1);
        assert!(!remove_utxo_pointers(&utxos_from_db_after).contains(&remove_utxo_pointer(&utxo_to_find)));
        remove_utxo_pointers(&utxos).iter().enumerate().for_each(|(i, utxo)| {
            if i != utxo_to_find_index {
                assert!(remove_utxo_pointers(&utxos_from_db_after).contains(utxo))
            }
        });
    }

    #[test]
    fn should_get_utxos_with_tx_id_and_v_out_correctly() {
        let utxos = get_sample_utxo_and_values();
        utxos
            .iter()
            .enumerate()
            .for_each(|(i, _)| should_get_utxo_with_tx_id_and_v_out_correctly(utxos.clone(), i));
    }

    #[test]
    fn should_fail_to_find_non_existent_utxo_correctly() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxo_to_find_index = 3;
        let utxos = get_sample_utxo_and_values();
        let utxo_to_find = utxos[utxo_to_find_index].clone();
        let non_existent_v_out = utxo_to_find.get_v_out().unwrap() + 1;
        let tx_id = utxo_to_find.get_tx_id().unwrap();
        let mut expected_utxos_from_db_after = utxos.clone();
        expected_utxos_from_db_after.remove(utxo_to_find_index);
        save_utxos_to_db(&db, &utxos).unwrap();
        let utxos_from_db_before = get_all_utxos_without_removing_from_db(&db).unwrap();
        assert_eq!(utxos_from_db_before.len(), utxos.len());
        let expected_err = format!(
            "Could not find UTXO with v_out: {} & tx_id: {}",
            non_existent_v_out, tx_id
        );
        match get_utxo_with_tx_id_and_v_out(&db, non_existent_v_out, &tx_id) {
            Ok(_) => panic!("Should not have found utxo!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(_) => panic!("Wrong error when finding non-existent utxo"),
        };
        let utxos_from_db_after = get_all_utxos_without_removing_from_db(&db).unwrap();
        assert_eq!(utxos_from_db_after.len(), utxos.len());
        remove_utxo_pointers(&utxos_from_db_after)
            .iter()
            .for_each(|utxo| assert!(remove_utxo_pointers(&utxos).contains(utxo)));
    }

    #[test]
    fn should_get_total_number_of_utxos_from_db() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        save_utxos_to_db(&db, &utxos).unwrap();
        let expected_result = utxos.len();
        let result = get_total_number_of_utxos_from_db(&db);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_x_utxos() {
        let num_utxos_to_get = 4;
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        save_utxos_to_db(&db, &utxos).unwrap();
        let result = get_x_utxos(&db, num_utxos_to_get).unwrap();
        assert_eq!(result.len(), num_utxos_to_get);
        let num_utxos_remaining = get_total_number_of_utxos_from_db(&db);
        assert_eq!(num_utxos_remaining, utxos.len() - num_utxos_to_get);
    }

    #[test]
    fn should_fail_to_get_x_utxos_correctly() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        let num_utxos_to_get = utxos.len() + 1;
        save_utxos_to_db(&db, &utxos).unwrap();
        let expected_err = format!(
            "Can't get {} UTXOS, there're only {} in the db!",
            num_utxos_to_get,
            utxos.len()
        );
        match get_x_utxos(&db, num_utxos_to_get) {
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(_) => panic!("Wrong error receieved!"),
            Ok(_) => panic!("Should not have succeeded!"),
        };
    }

    #[test]
    fn should_have_helpful_error_message_if_no_utxos_in_db() {
        let db = get_test_database();
        let expected_err_msg = "✘ No UTXOs in the database! Have you bricked this core?";
        match get_first_utxo_and_value(&db) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(err_msg)) => assert_eq!(err_msg, expected_err_msg),
            Err(_) => panic!("Wrong error received!"),
        };
    }

    #[test]
    fn utxo_exists_should_return_true_if_extant() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        save_utxos_to_db(&db, &utxos).unwrap();
        utxos.iter().for_each(|utxo| {
            let result = utxo_exists(&db, utxo.get_v_out().unwrap(), &utxo.get_tx_id().unwrap());
            assert!(result);
        })
    }

    #[test]
    fn utxo_exists_should_return_false_if_not_extant() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        let utxos_with_one_removed = BtcUtxosAndValues(utxos[..utxos.len() - 1].to_vec());
        assert_eq!(utxos.len(), utxos_with_one_removed.len() + 1);
        save_utxos_to_db(&db, &utxos_with_one_removed).unwrap();
        utxos.iter().enumerate().for_each(|(i, utxo)| {
            let result = utxo_exists(&db, utxo.get_v_out().unwrap(), &utxo.get_tx_id().unwrap());
            if i == utxos.len() - 1 {
                assert!(!result);
            } else {
                assert!(result);
            }
        })
    }

    #[test]
    fn should_not_save_same_utxo_twice() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        let utxos_with_duplicates = BtcUtxosAndValues([utxos.clone().0, utxos.clone().0].concat());
        assert_eq!(utxos_with_duplicates.len(), utxos.len() * 2);
        let utxo_nonce_before = get_utxo_nonce_from_db(&db).unwrap() as usize;
        assert_eq!(utxo_nonce_before, 0);
        let utxo_total_before = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(utxo_total_before, 0);
        let expected_utxo_nonce = utxos.len();
        save_utxos_to_db(&db, &utxos_with_duplicates).unwrap();
        let utxo_nonce_after = get_utxo_nonce_from_db(&db).unwrap() as usize;
        assert_eq!(utxo_nonce_after, expected_utxo_nonce);
        let utxo_total_after = get_total_utxo_balance_from_db(&db).unwrap();
        assert_eq!(utxo_total_after, utxos.sum());
    }
}
