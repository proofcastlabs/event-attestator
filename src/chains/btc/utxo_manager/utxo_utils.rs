use bitcoin_hashes::{
    Hash,
    sha256d,
};
use crate::{
    types::{
        Byte,
        Bytes,
        Result,
    },
    traits::DatabaseInterface,
    chains::btc::utxo_manager::{
        utxo_types::BtcUtxoAndValue,
        utxo_database_utils::{
            get_utxo_from_db,
            get_all_utxo_db_keys,
        },
    },
};

pub fn get_utxo_and_value_db_key(
    utxo_number: u64,
) -> Bytes {
    sha256d::Hash::hash(
        format!("utxo-number-{}", utxo_number).as_bytes()
    ).to_vec()
}

pub fn serialize_btc_utxo_and_value(
    btc_utxo_and_value: &BtcUtxoAndValue
) -> Result<Bytes> {
    Ok(serde_json::to_vec(btc_utxo_and_value)?)
}

pub fn deserialize_utxo_and_value(
    bytes: &[Byte]
) -> Result<BtcUtxoAndValue> {
    Ok(serde_json::from_slice(bytes)?)
}

pub fn get_all_utxos_as_json_string<D>(
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::btc::btc_test_utils::{
        get_sample_p2sh_utxo_and_value,
        get_sample_op_return_utxo_and_value,
    };

    #[test]
    fn should_serde_op_return_btc_utxo_and_value() {
        let utxo = get_sample_op_return_utxo_and_value();
        let serialized_utxo = serialize_btc_utxo_and_value(&utxo)
            .unwrap();
        let result = deserialize_utxo_and_value(&serialized_utxo)
            .unwrap();
        assert!(result == utxo);
    }

    #[test]
    fn should_serde_p2sh_btc_utxo_and_value() {
        let utxo = get_sample_p2sh_utxo_and_value()
            .unwrap();
        let serialized_utxo = serialize_btc_utxo_and_value(&utxo)
            .unwrap();
        let result = deserialize_utxo_and_value(&serialized_utxo)
            .unwrap();
        assert!(result == utxo);
    }

    #[test]
    fn should_get_utxo_db_key() {
        let expected_result =
            "b783e877488797a385ffd73089fc7d051db72ea1cf4290ee0d3a65efa712e29c";
        let num = 1;
        let result = get_utxo_and_value_db_key(num);
        assert!(hex::encode(result) == expected_result);
    }

    #[test]
    fn should_serde_utxo_and_value_with_something_in_the_maybe_pointer() {
        let mut utxo = get_sample_op_return_utxo_and_value();
        let pointer_hash = sha256d::Hash::hash(b"pointer hash");
        utxo.maybe_pointer = Some(pointer_hash);
        let serialized_utxo = serialize_btc_utxo_and_value(&utxo)
            .unwrap();
        let result = deserialize_utxo_and_value(&serialized_utxo)
            .unwrap();
        assert!(result == utxo);
    }
}
