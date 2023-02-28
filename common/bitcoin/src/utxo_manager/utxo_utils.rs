use bitcoin::{
    blockdata::transaction::{Transaction as BtcTransaction, TxIn as BtcUtxo},
    consensus::encode::deserialize as btc_deserialize,
    hashes::{sha256d, Hash},
};
use common::{
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};
use common_safe_addresses::SAFE_BTC_ADDRESS_STR;
use serde_json::{json, Value as JsonValue};

use crate::{
    btc_constants::{BTC_TX_LOCK_TIME, BTC_TX_VERSION, DUST_AMOUNT},
    btc_utils::create_new_pay_to_pub_key_hash_output,
    utxo_manager::{
        utxo_database_utils::{get_all_utxo_db_keys, get_first_utxo_and_value, get_utxo_from_db},
        utxo_types::{BtcUtxoAndValue, BtcUtxosAndValues},
    },
};

pub fn get_utxo_and_value_db_key(utxo_number: u64) -> Bytes {
    sha256d::Hash::hash(format!("utxo-number-{}", utxo_number).as_bytes()).to_vec()
}

pub fn serialize_btc_utxo_and_value(btc_utxo_and_value: &BtcUtxoAndValue) -> Result<Bytes> {
    Ok(serde_json::to_vec(btc_utxo_and_value)?)
}

pub fn deserialize_utxo_and_value(bytes: &[Byte]) -> Result<BtcUtxoAndValue> {
    Ok(serde_json::from_slice(bytes)?)
}

pub fn get_all_utxos_as_json_string<D: DatabaseInterface>(db: &D) -> Result<String> {
    db.start_transaction().and_then(|_| {
        let result = json!(get_all_utxo_db_keys(db)
            .iter()
            .map(|db_key| {
                get_utxo_from_db(db, db_key)
                    .and_then(|utxo_and_value| utxo_and_value.to_json())
                    .and_then(|utxo_and_value_json| {
                        Ok(json!({
                            "db_key": hex::encode(db_key),
                            "value": utxo_and_value_json.value,
                            "tx_id": utxo_and_value_json.tx_id,
                            "v_out": utxo_and_value_json.v_out,
                            "maybe_pointer": utxo_and_value_json.maybe_pointer,
                            "serialized_utxo": utxo_and_value_json.serialized_utxo,
                            "maybe_extra_data": utxo_and_value_json.maybe_extra_data,
                            "maybe_deposit_info_json": utxo_and_value_json.maybe_deposit_info_json,
                            "db_value": hex::encode(db.get(db_key.to_vec(), MIN_DATA_SENSITIVITY_LEVEL)?),
                        }))
                    })
            })
            .collect::<Result<Vec<JsonValue>>>()?)
        .to_string();
        db.end_transaction()?;
        Ok(result)
    })
}

fn get_all_utxos_from_db<D: DatabaseInterface>(db: &D) -> Result<Vec<BtcUtxoAndValue>> {
    get_all_utxo_db_keys(db)
        .iter()
        .map(|db_key| get_utxo_from_db(db, db_key))
        .collect()
}

fn get_btc_utxos_from_utxo_and_values(utxo_and_values: Vec<BtcUtxoAndValue>) -> Result<Vec<BtcUtxo>> {
    utxo_and_values
        .iter()
        .map(|utxo_and_value| Ok(btc_deserialize(&utxo_and_value.serialized_utxo)?))
        .collect::<Result<Vec<BtcUtxo>>>()
}

pub fn utxos_exist_in_db<D: DatabaseInterface>(db: &D, utxos_to_check: &BtcUtxosAndValues) -> Result<Vec<bool>> {
    debug!("✔ Checking if UTXOs exist in db...");
    get_all_utxos_from_db(db)
        .and_then(get_btc_utxos_from_utxo_and_values)
        .and_then(|btc_utxos_from_db| {
            utxos_to_check
                .0
                .iter()
                .map(|utxo_and_value| -> Result<BtcUtxo> { Ok(btc_deserialize(&utxo_and_value.serialized_utxo)?) })
                .map(|utxo| -> Result<bool> { Ok(btc_utxos_from_db.contains(&utxo?)) })
                .collect()
        })
}

pub fn get_enough_utxos_to_cover_total<D: DatabaseInterface>(
    db: &D,
    sats_required: u64,
    num_outputs: usize,
    sats_per_byte: u64,
) -> Result<BtcUtxosAndValues> {
    get_enough_utxos_to_cover_total_recursively(db, sats_required, num_outputs, sats_per_byte, vec![].into())
}

fn get_enough_utxos_to_cover_total_recursively<D: DatabaseInterface>(
    db: &D,
    sats_required: u64,
    num_outputs: usize,
    sats_per_byte: u64,
    mut inputs: BtcUtxosAndValues,
) -> Result<BtcUtxosAndValues> {
    // NOTE: This function assumes the CALLER is accounting for a change output!
    info!("✔ Getting UTXO from db...");
    let utxo = get_first_utxo_and_value(db)?;
    debug!("✔ Retrieved UTXO of value: {}", utxo.value);

    // NOTE: Add the UTXO to the inputs array...
    inputs.push(utxo);

    // NOTE: Create the correct number of outputs for the transaction...
    let mut dummy_outputs = vec![];
    for _ in 1..=num_outputs {
        let output = create_new_pay_to_pub_key_hash_output(0, SAFE_BTC_ADDRESS_STR)?;
        dummy_outputs.push(output)
    }

    // NOTE: Create a dummy tx so we can correctly calculate the size & thus the fee...
    let dummy_tx = BtcTransaction {
        output: dummy_outputs,
        version: BTC_TX_VERSION,
        input: inputs.to_utxos()?,
        lock_time: BTC_TX_LOCK_TIME,
    };
    let fee = dummy_tx.size() as u64 * sats_per_byte;

    // NOTE: Calculate total + fee to check if we have enough UTXOs to cover it...
    let total_cost = fee + sats_required;
    let total_utxo_value = inputs.sum();
    debug!(
        "✔ Calculated fee for {} input(s) & {} output(s): {} satoshis",
        inputs.len(),
        num_outputs,
        fee
    );
    debug!("✔ Fee + required BTC value of tx: {} Satoshis", total_cost);
    debug!("✔ Current total UTXO value: {} Satoshis", total_utxo_value);
    if total_cost <= total_utxo_value {
        // NOTE: Now we can safely subtract and find the change amount without underflowing...
        let change_amount = total_utxo_value - total_cost;
        debug!("✔ Dust amount: {} Satoshis", *DUST_AMOUNT);
        debug!("✔ Change amount: {} Satoshis", change_amount);
        // NOTE: And finally check if the change output will be dust or not!
        if change_amount <= *DUST_AMOUNT {
            debug!("Change UTXO will be dust, we need another!");
            get_enough_utxos_to_cover_total_recursively(db, sats_required, num_outputs, sats_per_byte, inputs)
        } else {
            debug!("✔ UTXO(s) covers fee + required satoshi amount, and the change amount is not dust!");
            Ok(inputs)
        }
    } else {
        debug!("UTXOs do not cover fee + amount, we need another!");
        get_enough_utxos_to_cover_total_recursively(db, sats_required, num_outputs, sats_per_byte, inputs)
    }
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::{
        test_utils::{get_sample_p2pkh_utxo_and_value, get_sample_p2sh_utxo_and_value, get_sample_utxo_and_values},
        utxo_manager::utxo_database_utils::{save_new_utxo_and_value, save_utxos_to_db, set_utxo_balance_to_zero},
    };

    #[test]
    fn should_serde_p2pkh_btc_utxo_and_value() {
        let utxo = get_sample_p2pkh_utxo_and_value();
        let serialized_utxo = serialize_btc_utxo_and_value(&utxo).unwrap();
        let result = deserialize_utxo_and_value(&serialized_utxo).unwrap();
        assert_eq!(result, utxo);
    }

    #[test]
    fn should_serde_p2sh_btc_utxo_and_value() {
        let utxo = get_sample_p2sh_utxo_and_value().unwrap();
        let serialized_utxo = serialize_btc_utxo_and_value(&utxo).unwrap();
        let result = deserialize_utxo_and_value(&serialized_utxo).unwrap();
        assert_eq!(result, utxo);
    }

    #[test]
    fn should_get_utxo_db_key() {
        let expected_result = "b783e877488797a385ffd73089fc7d051db72ea1cf4290ee0d3a65efa712e29c";
        let num = 1;
        let result = get_utxo_and_value_db_key(num);
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_serde_utxo_and_value_with_something_in_the_maybe_pointer() {
        let mut utxo = get_sample_p2pkh_utxo_and_value();
        let pointer_hash = sha256d::Hash::hash(b"pointer hash");
        utxo.maybe_pointer = Some(pointer_hash);
        let serialized_utxo = serialize_btc_utxo_and_value(&utxo).unwrap();
        let result = deserialize_utxo_and_value(&serialized_utxo).unwrap();
        assert_eq!(result, utxo);
    }

    #[test]
    fn should_return_correct_bool_array_when_checking_it_multiple_utxos_exist_in_db() {
        let expected_result = vec![false, true];
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxo_and_value_1 = get_sample_p2sh_utxo_and_value().unwrap();
        let utxo_and_value_2 = get_sample_p2pkh_utxo_and_value();
        save_new_utxo_and_value(&db, &utxo_and_value_2).unwrap();
        let result = utxos_exist_in_db(&db, &BtcUtxosAndValues::new(vec![utxo_and_value_1, utxo_and_value_2])).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_all_utxos_as_json_string() {
        let db = get_test_database();
        set_utxo_balance_to_zero(&db).unwrap();
        let utxos = get_sample_utxo_and_values();
        save_utxos_to_db(&db, &utxos).unwrap();
        let result = get_all_utxos_as_json_string(&db);
        assert!(result.is_ok());
    }
}
