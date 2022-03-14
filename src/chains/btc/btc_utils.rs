use std::str::FromStr;

use bitcoin::{
    blockdata::{
        opcodes,
        script::{Builder as BtcScriptBuilder, Script as BtcScript},
        transaction::{OutPoint as BtcOutPoint, Transaction as BtcTransaction, TxIn as BtcUtxo, TxOut as BtcTxOut},
    },
    consensus::encode::{deserialize as btc_deserialize, serialize as btc_serialize},
    hash_types::Txid,
    hashes::{sha256d, Hash},
    network::constants::Network as BtcNetwork,
    secp256k1::key::ONE_KEY,
    util::{
        base58::{encode_slice as base58_encode_slice, from as from_base58},
        key::PrivateKey,
    },
    Address as BtcAddress,
};
use ethereum_types::U256;

use crate::{
    chains::btc::{
        btc_constants::{BTC_PUB_KEY_SLICE_LENGTH, BTC_TX_LOCK_TIME, BTC_TX_VERSION, DEFAULT_BTC_SEQUENCE},
        btc_types::BtcPubKeySlice,
    },
    constants::{BTC_NUM_DECIMALS, PTOKEN_ERC777_NUM_DECIMALS},
    safe_addresses::SAFE_BTC_ADDRESS,
    types::{Byte, Bytes, Result},
    utils::strip_hex_prefix,
};

pub fn calculate_dust_amount(dust_relay_fee: u64) -> u64 {
    // NOTE: See https://bitcoin.stackexchange.com/questions/10986/what-is-meant-by-bitcoin-dust
    const PUB_KEY_HASH_SIZE_IN_BYTES: usize = 20;
    // NOTE: This is destinated for a dummy tx, so we don't care what the address is!
    let dummy_pub_key_hash_bytes = [0u8; PUB_KEY_HASH_SIZE_IN_BYTES];
    let script_sig = get_pay_to_pub_key_hash_script_from_slice(&dummy_pub_key_hash_bytes);
    let output = BtcTxOut {
        value: 0,
        script_pubkey: script_sig.clone(),
    };
    // NOTE: Now we create the dummy tx that would spend a change output of the type that the core
    // would write...
    let dummy_tx = BtcTransaction {
        version: BTC_TX_VERSION,
        lock_time: BTC_TX_LOCK_TIME,
        output: vec![output],
        input: vec![BtcUtxo {
            script_sig,
            witness: Vec::default(),
            sequence: u32::default(),
            previous_output: BtcOutPoint::default(),
        }],
    };
    // NOTE: Then we calculate the size of that transaction...
    let dummy_tx_size_in_bytes = dummy_tx.get_size() as u64;
    // NOTE: Which we use we calculate the minimum allowable fee to spend this output...
    let cost_to_spend_utxo = dust_relay_fee * dummy_tx_size_in_bytes;
    // NOTE: And so dust is any amount whose fee to spend it is > 1/3 of the value of the UTXO itself.
    let dust_amount = cost_to_spend_utxo * 3;
    debug!("Calculated dust amount: {}", dust_amount);
    dust_amount
}

pub fn convert_hex_tx_to_btc_transaction(hex: String) -> Result<BtcTransaction> {
    Ok(btc_deserialize::<BtcTransaction>(&hex::decode(hex)?)?)
}

pub fn convert_bytes_to_btc_pub_key_slice(bytes: &[Byte]) -> Result<BtcPubKeySlice> {
    match bytes.len() {
        0..=32 => Err("✘ Too few bytes to convert to BTC pub key slice!".into()),
        BTC_PUB_KEY_SLICE_LENGTH => {
            let mut arr = [0u8; BTC_PUB_KEY_SLICE_LENGTH];
            let bytes = &bytes[..BTC_PUB_KEY_SLICE_LENGTH];
            arr.copy_from_slice(bytes);
            Ok(arr)
        },
        _ => Err("✘ Too many bytes to convert to BTC pub key slice!".into()),
    }
}

pub fn convert_hex_to_sha256_hash(hex: &str) -> Result<sha256d::Hash> {
    Ok(sha256d::Hash::from_slice(&hex::decode(strip_hex_prefix(hex))?)?)
}

pub fn get_btc_one_key() -> PrivateKey {
    PrivateKey {
        key: ONE_KEY,
        compressed: false,
        network: BtcNetwork::Bitcoin,
    }
}

pub fn get_p2sh_redeem_script_sig(utxo_spender_pub_key_slice: &[u8], commitment_hash: &sha256d::Hash) -> BtcScript {
    info!("✔ Generating `p2sh`'s redeem `script_sig`");
    debug!("✔ Using `commitment_hash`: {}", hex::encode(commitment_hash));
    debug!("✔ Using `pub key slice`: {}", hex::encode(utxo_spender_pub_key_slice));
    BtcScriptBuilder::new()
        .push_slice(&commitment_hash[..])
        .push_opcode(opcodes::all::OP_DROP)
        .push_slice(utxo_spender_pub_key_slice)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
}

pub fn get_p2sh_script_sig_from_redeem_script(signature_slice: &[u8], redeem_script: &BtcScript) -> BtcScript {
    BtcScriptBuilder::new()
        .push_slice(signature_slice)
        .push_slice(redeem_script.as_bytes())
        .into_script()
}

pub fn create_unsigned_utxo_from_tx(tx: &BtcTransaction, output_index: u32) -> BtcUtxo {
    let outpoint = BtcOutPoint {
        txid: tx.txid(),
        vout: output_index,
    };
    BtcUtxo {
        witness: vec![], // NOTE: We don't currently support segwit txs.
        previous_output: outpoint,
        sequence: DEFAULT_BTC_SEQUENCE,
        script_sig: tx.output[output_index as usize].script_pubkey.clone(),
    }
}

pub fn get_hex_tx_from_signed_btc_tx(signed_btc_tx: &BtcTransaction) -> String {
    hex::encode(btc_serialize(signed_btc_tx))
}

pub fn get_script_sig<'a>(signature_slice: &'a [u8], utxo_spender_pub_key_slice: &'a [u8]) -> BtcScript {
    let script_builder = BtcScriptBuilder::new();
    script_builder
        .push_slice(signature_slice)
        .push_slice(utxo_spender_pub_key_slice)
        .into_script()
}

pub fn create_new_tx_output(value: u64, script: BtcScript) -> BtcTxOut {
    BtcTxOut {
        value,
        script_pubkey: script,
    }
}

pub fn create_new_pay_to_pub_key_hash_output(value: u64, recipient: &str) -> Result<BtcTxOut> {
    get_pay_to_pub_key_hash_script(recipient).map(|script| create_new_tx_output(value, script))
}

pub fn serialize_btc_utxo(btc_utxo: &BtcUtxo) -> Bytes {
    btc_serialize(btc_utxo)
}

pub fn deserialize_btc_utxo(bytes: &[Byte]) -> Result<BtcUtxo> {
    Ok(btc_deserialize(bytes)?)
}

pub fn convert_btc_address_to_bytes(btc_address: &str) -> Result<Bytes> {
    Ok(from_base58(btc_address)?)
}

pub fn convert_bytes_to_btc_address(encoded_bytes: Bytes) -> String {
    base58_encode_slice(&encoded_bytes)
}

pub fn convert_btc_address_to_pub_key_hash_bytes(btc_address: &str) -> Result<Bytes> {
    Ok(from_base58(btc_address)?[1..21].to_vec())
}

pub fn get_pay_to_pub_key_hash_script(btc_address: &str) -> Result<BtcScript> {
    convert_btc_address_to_pub_key_hash_bytes(btc_address)
        .map(|ref bytes| get_pay_to_pub_key_hash_script_from_slice(bytes))
}

pub fn get_pay_to_pub_key_hash_script_from_slice(slice: &[u8]) -> BtcScript {
    let script = BtcScriptBuilder::new();
    script
        .push_opcode(opcodes::all::OP_DUP)
        .push_opcode(opcodes::all::OP_HASH160)
        .push_slice(slice)
        .push_opcode(opcodes::all::OP_EQUALVERIFY)
        .push_opcode(opcodes::all::OP_CHECKSIG)
        .into_script()
}

pub fn get_btc_tx_id_from_str(tx_id: &str) -> Result<Txid> {
    match hex::decode(tx_id) {
        Err(_) => Err("Could not decode tx_id hex string!".into()),
        Ok(mut bytes) => {
            // NOTE: Weird endianess switch quirk of how BTC displays Txids:
            // NOTE: https://bitcoin.stackexchange.com/questions/39363/compute-txid-of-bitcoin-transaction
            bytes.reverse();
            Ok(Txid::from_slice(&bytes)?)
        },
    }
}

pub fn convert_str_to_btc_address_or_safe_address(s: &str) -> Result<BtcAddress> {
    match BtcAddress::from_str(s) {
        Ok(address) => Ok(address),
        _ => {
            info!(
                "✔ Could not parse BTC from string: '{}'! Diverting to safe BTC address...",
                s
            );
            let safe_address = SAFE_BTC_ADDRESS.clone();
            Ok(safe_address)
        },
    }
}

#[cfg(test)] // TODO Create then move this to chains/btc_test_utils!
pub fn get_tx_id_from_signed_btc_tx(signed_btc_tx: &BtcTransaction) -> String {
    let mut tx_id = signed_btc_tx.txid().to_vec();
    tx_id.reverse();
    hex::encode(tx_id)
}

pub fn convert_satoshis_to_wei(satoshis: u64) -> U256 {
    U256::from(satoshis) * U256::from(10u64.pow(PTOKEN_ERC777_NUM_DECIMALS - BTC_NUM_DECIMALS as u32))
}

pub fn convert_wei_to_satoshis(ptoken: U256) -> u64 {
    match ptoken.checked_div(U256::from(
        10u64.pow(PTOKEN_ERC777_NUM_DECIMALS - BTC_NUM_DECIMALS as u32),
    )) {
        Some(amount) => amount.as_u64(),
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bitcoin::hashes::{sha256d, Hash};

    use super::*;
    use crate::{
        chains::btc::{
            btc_test_utils::{
                create_p2pkh_btc_utxo_and_value_from_tx_output,
                get_sample_btc_block_and_id,
                get_sample_btc_private_key,
                get_sample_btc_pub_key_slice,
                get_sample_btc_utxo,
                get_sample_p2pkh_utxo_and_value_n,
                get_sample_p2sh_redeem_script_sig,
                get_sample_testnet_block_and_txs,
                SAMPLE_OUTPUT_INDEX_OF_UTXO,
                SAMPLE_SERIALIZED_BTC_UTXO,
                SAMPLE_TARGET_BTC_ADDRESS,
                SAMPLE_TRANSACTION_INDEX,
            },
            utxo_manager::utxo_types::BtcUtxosAndValues,
        },
        errors::AppError,
    };

    #[test]
    fn should_convert_satoshis_to_wei() {
        let satoshis = 1337;
        let expected_result = U256::from_dec_str("13370000000000").unwrap();
        let result = convert_satoshis_to_wei(satoshis);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_wei_to_satoshis() {
        let ptoken = U256::from_dec_str("13370000000000").unwrap();
        let expected_result = 1337;
        let result = convert_wei_to_satoshis(ptoken);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_create_new_pay_to_pub_key_hash_output() {
        let expected_script = get_pay_to_pub_key_hash_script(SAMPLE_TARGET_BTC_ADDRESS).unwrap();
        let value = 1;
        let result = create_new_pay_to_pub_key_hash_output(value, SAMPLE_TARGET_BTC_ADDRESS).unwrap();
        assert_eq!(result.value, value);
        assert_eq!(result.script_pubkey, expected_script);
    }

    #[test]
    fn should_create_new_tx_output() {
        let value = 1;
        let script = get_pay_to_pub_key_hash_script(SAMPLE_TARGET_BTC_ADDRESS).unwrap();
        let result = create_new_tx_output(value, script.clone());
        assert_eq!(result.value, value);
        assert_eq!(result.script_pubkey, script);
    }

    #[test]
    fn should_serialize_btc_utxo() {
        let result = hex::encode(serialize_btc_utxo(&get_sample_btc_utxo()));
        assert_eq!(result, SAMPLE_SERIALIZED_BTC_UTXO);
    }

    #[test]
    fn should_deserialize_btc_utxo() {
        let expected_vout = SAMPLE_OUTPUT_INDEX_OF_UTXO;
        let expected_witness_length = 0;
        let expected_sequence = 4294967295;
        let expected_txid = Txid::from_str("04bf43a86a99fca519dbfce42566b78cda0895d78c0a07484162d5888f588d0e").unwrap();
        let serialized_btc_utxo = hex::decode(SAMPLE_SERIALIZED_BTC_UTXO).unwrap();
        let result = deserialize_btc_utxo(&serialized_btc_utxo).unwrap();
        assert_eq!(result.sequence, expected_sequence);
        assert_eq!(result.previous_output.txid, expected_txid);
        assert_eq!(result.previous_output.vout, expected_vout);
        assert_eq!(result.witness.len(), expected_witness_length);
    }

    #[test]
    fn should_convert_btc_address_to_bytes() {
        let expected_result_hex = "6f54102783c8640c5144d039cea53eb7dbb470081462fbafd9";
        let result = convert_btc_address_to_bytes(&SAMPLE_TARGET_BTC_ADDRESS.to_string()).unwrap();
        let result_hex = hex::encode(result);
        assert_eq!(result_hex, expected_result_hex);
    }

    #[test]
    fn should_convert_bytes_to_btc_address() {
        let bytes = convert_btc_address_to_bytes(&SAMPLE_TARGET_BTC_ADDRESS.to_string()).unwrap();
        let result = convert_bytes_to_btc_address(bytes);
        assert_eq!(result, SAMPLE_TARGET_BTC_ADDRESS);
    }

    #[test]
    fn should_convert_btc_address_to_pub_key_hash_bytes() {
        let expected_result = "54102783c8640c5144d039cea53eb7dbb4700814";
        let result = convert_btc_address_to_pub_key_hash_bytes(SAMPLE_TARGET_BTC_ADDRESS).unwrap();
        assert_eq!(hex::encode(result), expected_result);
    }

    #[test]
    fn should_get_pay_to_pub_key_hash_script() {
        let example_script = get_sample_testnet_block_and_txs().unwrap().block.txdata
            [SAMPLE_TRANSACTION_INDEX as usize]
            .output[SAMPLE_OUTPUT_INDEX_OF_UTXO as usize]
            .script_pubkey
            .clone();
        let expected_result = "76a91454102783c8640c5144d039cea53eb7dbb470081488ac";
        let result_script = get_pay_to_pub_key_hash_script(SAMPLE_TARGET_BTC_ADDRESS).unwrap();
        let hex_result = hex::encode(result_script.as_bytes());
        assert!(!result_script.is_p2sh());
        assert!(result_script.is_p2pkh());
        assert_eq!(hex_result, expected_result);
        assert_eq!(result_script, example_script);
    }

    #[test]
    fn should_get_script_sig() {
        let expected_result = "4730440220275e800c20aa5096a49e6c36aae8f532093fc3fdc4a1dd6039314b250efd62300220492fe4b7e27bf555648f023811fb2258bbcd057fd54967f96942cf1f606e4fe7012103d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7";
        let hash_type = 1;
        let hash = sha256d::Hash::hash(b"a message");
        let btc_pk = get_sample_btc_private_key();
        let signature = btc_pk
            .sign_hash_and_append_btc_hash_type(hash.to_vec(), hash_type)
            .unwrap();
        let pub_key_slice = get_sample_btc_pub_key_slice();
        let result_script = get_script_sig(&signature, &pub_key_slice);
        let hex_result = hex::encode(result_script.as_bytes());
        assert_eq!(hex_result, expected_result);
    }

    #[test]
    fn should_get_total_value_of_utxos_and_values() {
        let expected_result = 1942233;
        let utxos = BtcUtxosAndValues::new(vec![
            get_sample_p2pkh_utxo_and_value_n(2).unwrap(),
            get_sample_p2pkh_utxo_and_value_n(3).unwrap(),
            get_sample_p2pkh_utxo_and_value_n(4).unwrap(),
        ]);
        let result = utxos.iter().fold(0, |acc, utxo_and_value| acc + utxo_and_value.value);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_get_p2sh_redeem_script_sig() {
        let result = get_sample_p2sh_redeem_script_sig();
        let result_hex = hex::encode(result.as_bytes());
        let expected_result = "2071a8e55edefe53f703646a679e66799cfef657b98474ff2e4148c3a1ea43169c752103d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7ac";
        assert_eq!(result_hex, expected_result);
    }

    #[test]
    fn should_get_p2sh_script_sig_from_redeem_script() {
        let signature_slice = &vec![6u8, 6u8, 6u8][..];
        let redeem_script = get_sample_p2sh_redeem_script_sig();
        let expected_result = "03060606452071a8e55edefe53f703646a679e66799cfef657b98474ff2e4148c3a1ea43169c752103d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7ac";
        let result = get_p2sh_script_sig_from_redeem_script(&signature_slice, &redeem_script);
        let result_hex = hex::encode(result.as_bytes());
        assert_eq!(result_hex, expected_result);
    }

    #[test]
    fn should_create_unsigned_utxo_from_tx() {
        let expected_result = "f80c2f7c35f5df8441a5a5b52e2820793fc7e69f4603d38ba7217be41c20691d0000000016001497cfc76442fe717f2a3f0cc9c175f7561b661997ffffffff";
        let index = 0;
        let tx = get_sample_btc_block_and_id().unwrap().block.txdata[0].clone();
        let result = create_unsigned_utxo_from_tx(&tx, index);
        let result_hex = hex::encode(btc_serialize(&result));
        assert_eq!(result_hex, expected_result);
    }

    #[test]
    fn should_create_p2pkh_btc_utxo_and_value_from_tx_output() {
        let expected_value = 1261602424;
        let expected_utxo = "f80c2f7c35f5df8441a5a5b52e2820793fc7e69f4603d38ba7217be41c20691d0000000016001497cfc76442fe717f2a3f0cc9c175f7561b661997ffffffff";
        let index = 0;
        let tx = get_sample_btc_block_and_id().unwrap().block.txdata[0].clone();
        let result = create_p2pkh_btc_utxo_and_value_from_tx_output(&tx, index);
        assert_eq!(result.maybe_pointer, None);
        assert_eq!(result.value, expected_value);
        assert_eq!(result.maybe_extra_data, None);
        assert_eq!(result.maybe_deposit_info_json, None);
        assert_eq!(hex::encode(result.serialized_utxo), expected_utxo);
    }

    #[test]
    fn should_convert_bytes_to_btc_pub_key_slice() {
        let bytes = hex::decode("03a3bea6d8d15a38d9c96074d994c788bc1286d557ef5bdbb548741ddf265637ce").unwrap();
        let result = convert_bytes_to_btc_pub_key_slice(&bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_to_convert_too_short_bytes_to_btc_pub_key_slice_correctly() {
        let expected_err = "✘ Too few bytes to convert to BTC pub key slice!".to_string();
        let bytes = hex::decode("03a3bea6d8d15a38d9c96074d994c788bc1286d557ef5bdbb548741ddf265637").unwrap();
        match convert_bytes_to_btc_pub_key_slice(&bytes) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(_) => panic!("Got wrong error when failing to convert bytes to `BtcPubKeySlice`!"),
        }
    }

    #[test]
    fn should_fail_to_convert_too_long_bytes_to_btc_pub_key_slice_correctly() {
        let expected_err = "✘ Too many bytes to convert to BTC pub key slice!".to_string();
        let bytes = hex::decode("03a3bea6d8d15a38d9c96074d994c788bc1286d557ef5bdbb548741ddf265637abab").unwrap();
        match convert_bytes_to_btc_pub_key_slice(&bytes) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(_) => panic!("Got wrong error when failing to convert bytes to `BtcPubKeySlice`!"),
        }
    }

    #[test]
    fn should_get_btc_id_from_str() {
        let tx_id = "2704c7318a189ea87ec68c101fe3e17aaa62e5f5ede30f43a018301ee814e348";
        let mut bytes = hex::decode(tx_id).unwrap();
        bytes.reverse();
        let result = get_btc_tx_id_from_str(tx_id).unwrap();
        let expected_result = Txid::from_slice(&bytes).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_str_to_btc_address_or_safe_address() {
        let good_btc_address_str = "136CTERaocm8dLbEtzCaFtJJX9jfFhnChK";
        let result = convert_str_to_btc_address_or_safe_address(good_btc_address_str).unwrap();
        let expected_result = BtcAddress::from_str(good_btc_address_str).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_revert_to_safe_btc_address_if_it_cannot_convert_str_to_btc_address() {
        let bad_btc_address_str = "not a BTC address";
        let result = convert_str_to_btc_address_or_safe_address(bad_btc_address_str).unwrap();
        let expected_result = SAFE_BTC_ADDRESS.clone();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_calculate_dust_amount() {
        let expected_result = 990;
        let dust_relay_fee = 3;
        let result = calculate_dust_amount(dust_relay_fee);
        assert_eq!(result, expected_result);
    }
}
