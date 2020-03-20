use eos_primitives::AccountName as EosAccountName;
use std::{
    str::FromStr,
    fs::read_to_string,
};
use bitcoin::{
    hashes::{
        Hash,
        sha256d,
    },
    blockdata::{
        script::Script as BtcScript,
        transaction::{
            TxIn as BtcUtxo,
            TxOut as BtcTxOut,
            OutPoint as BtcOutPoint,
            Transaction as BtcTransaction,
        },
    },
};
use crate::btc_on_eos::{
    errors::AppError,
    utils::convert_u64_to_eos_asset,
    constants::MINIMUM_REQUIRED_SATOSHIS,
    types::{
        Bytes,
        Result,
    },
    btc::{
        btc_utils::{
            get_p2sh_redeem_script_sig,
            get_btc_block_in_db_format,
            create_unsigned_utxo_from_tx,
            get_pay_to_pub_key_hash_script,
            create_op_return_btc_utxo_and_value_from_tx_output,
        },
        btc_crypto::btc_private_key::BtcPrivateKey,
        btc_types::{
            MintingParams,
            BtcBlockAndId,
            BtcUtxoAndValue,
            MintingParamStruct,
            BtcBlockInDbFormat,
            DepositAddressInfoJson,
            SubmissionMaterialJson,
        },
        parse_submission_material::{
            parse_submission_material_to_json,
            parse_btc_block_from_submission_material,
        },
    },
};

pub const SAMPLE_TRANSACTION_INDEX: usize = 1;
pub const SAMPLE_BTC_UTXO_VALUE: u64 = 3347338;
pub const SAMPLE_OUTPUT_INDEX_OF_UTXO: u32 = 0;
pub const SAMPLE_TRANSACTION_OUTPUT_INDEX: usize = 0;
pub const SAMPLE_OP_RETURN_TRANSACTION_INDEX: usize = 56;
pub const SAMPLE_OP_RETURN_TRANSACTION_OUTPUT_INDEX: usize = 1;

pub const SAMPLE_TARGET_BTC_ADDRESS: &'static str =
    "moBSQbHn7N9BC9pdtAMnA7GBiALzNMQJyE";

pub const SAMPLE_BTC_BLOCK_JSON_PATH: &str =
    "src/btc_on_eos/btc/btc_test_utils/604700-btc-block-and-txs.json";

pub const SAMPLE_BTC_PRIVATE_KEY_WIF: &'static str =
    "cP2Dv4mx1DwJzN8iF6CCyPZmuS27bT9MV4Qmgb9h6cNQNq2Jgpmy";

pub const SAMPLE_BTC_PUBLIC_KEY: &'static str =
    "03d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH: &str =
    "src/btc_on_eos/btc/btc_test_utils/1610046-testnet-block-with-tx-to-test-address.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_2: &str =
    "src/btc_on_eos/btc/btc_test_utils/1610166-testnet-block-with-tx-to-test-address.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_3: &str =
    "src/btc_on_eos/btc/btc_test_utils/1610161-testnet-block-with-tx-to-test-address.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_5: &str =
    "src/btc_on_eos/btc/btc_test_utils/1637173-testnet-block-and-txs-with-p2sh-deposit.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_6: &str =
    "src/btc_on_eos/btc/btc_test_utils/1660807-testnet-block-and-txs-with-2-p2sh-deposits.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_7: &str =
    "src/btc_on_eos/btc/btc_test_utils/btc-1661479-btc-block-and-txs-with-deposit-originating-from-enclave-key.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_8: &str =
    "src/btc_on_eos/btc/btc_test_utils/1661611-block-and-txs-with-no-op-return.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_9: &str =
    "src/btc_on_eos/btc/btc_test_utils/1666951-block-and-txs-p2sh-and-op-return-below-threshold.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_10: &str =
    "src/btc_on_eos/btc/btc_test_utils/btc-1670411-block-and-txs.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_11: &str =
    "src/btc_on_eos/btc/btc_test_utils/btc-1670534-block-and-txs.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_12: &str =
    "src/btc_on_eos/btc/btc_test_utils/btc-1670541-block-and-txs.json";

pub const SAMPLE_TESTNET_OP_RETURN_BTC_BLOCK_JSON: &str =
    "src/btc_on_eos/btc/btc_test_utils/1610826-testnet-block-with-tx-to-test-address.json";

pub const SAMPLE_SERIALIZED_BTC_UTXO: &'static str = "0e8d588f88d5624148070a8cd79508da8cb76625e4fcdb19a5fc996aa843bf04000000001976a91454102783c8640c5144d039cea53eb7dbb470081488acffffffff";

pub fn get_sample_btc_pub_key_bytes() -> Bytes {
    hex::decode(SAMPLE_BTC_PUBLIC_KEY).unwrap()
}

pub fn get_sample_minting_params() -> MintingParams {
    let originating_tx_address_1 = "eosaccount1x".to_string();
    let originating_tx_address_2 = "eosaccount2x".to_string();
    let originating_tx_address_3 = "eosaccount3x".to_string();
    let eos_address_1 = originating_tx_address_1.clone();
    let eos_address_2 = originating_tx_address_2.clone();
    let eos_address_3 = originating_tx_address_3.clone();
    let amount_1 = convert_u64_to_eos_asset(MINIMUM_REQUIRED_SATOSHIS);
    let amount_2 = convert_u64_to_eos_asset(MINIMUM_REQUIRED_SATOSHIS + 1);
    let amount_3 = convert_u64_to_eos_asset(MINIMUM_REQUIRED_SATOSHIS - 1);
    let originating_tx_hash_1 = sha256d::Hash::hash(b"something_1").to_string();
    let originating_tx_hash_2 = sha256d::Hash::hash(b"something_2").to_string();
    let originating_tx_hash_3 = sha256d::Hash::hash(b"something_3").to_string();
    let minting_params_1 = MintingParamStruct {
        amount: amount_1,
        to: eos_address_1,
        originating_tx_hash: originating_tx_hash_1,
        originating_tx_address: originating_tx_address_1.clone(),
    };
    let minting_params_2 = MintingParamStruct {
        amount: amount_2,
        to: eos_address_2,
        originating_tx_hash: originating_tx_hash_2,
        originating_tx_address: originating_tx_address_2.clone(),
    };
    let minting_params_3 = MintingParamStruct {
        amount: amount_3,
        to: eos_address_3,
        originating_tx_hash: originating_tx_hash_3,
        originating_tx_address: originating_tx_address_3.clone(),
    };
    vec![minting_params_1, minting_params_2, minting_params_3]
}

pub fn get_sample_sequential_btc_blocks_in_db_format(
) -> Vec<BtcBlockInDbFormat> {
    let start_num = 1611090;
    let path_prefix = "src/btc_on_eos/btc/btc_test_utils/sequential_block_and_ids/";
    let path_suffix = "-btc-block-and-txs.json";
    let mut paths = Vec::new();
    for i in 0..11 {
        paths.push(
            format!("{}{}{}", path_prefix, start_num + i, path_suffix)
        )
    };
    paths
        .iter()
        .map(|path|
             read_to_string(path).unwrap()
        )
        .map(|json_string|
            parse_submission_material_to_json(&json_string)
                .and_then(|json|
                    parse_btc_block_from_submission_material(&json)
                )
                .and_then(convert_sample_block_to_db_format)
        )
        .collect::<Result<Vec<BtcBlockInDbFormat>>>()
        .unwrap()
}

pub fn get_sample_p2sh_redeem_script_sig() -> BtcScript {
    let pub_key_slice = get_sample_btc_private_key()
        .to_public_key_slice();
    let hash = sha256d::Hash::hash(b"some text");
    get_p2sh_redeem_script_sig(&pub_key_slice, &hash)
}

pub fn convert_sample_block_to_db_format(
    btc_block_and_id: BtcBlockAndId,
) -> Result<BtcBlockInDbFormat> {
    get_btc_block_in_db_format(
        btc_block_and_id,
        Vec::new(),
        Vec::new(),
    )
}

pub fn get_sample_btc_block_json_string() -> String {
    read_to_string(SAMPLE_BTC_BLOCK_JSON_PATH)
        .unwrap()
}

pub fn get_sample_btc_block_json() -> Result<SubmissionMaterialJson> {
    parse_submission_material_to_json(
        &get_sample_btc_block_json_string()
    )
}

pub fn get_sample_btc_block_and_id() -> Result<BtcBlockAndId> {
    parse_btc_block_from_submission_material(
        &get_sample_btc_block_json()
            .unwrap()
    )
}

pub fn get_sample_btc_block_in_db_format() -> Result<BtcBlockInDbFormat> {
    get_sample_btc_block_and_id()
        .and_then(convert_sample_block_to_db_format)
}

pub fn get_sample_testnet_block_and_txs() -> Result<BtcBlockAndId> {
    parse_submission_material_to_json(
        &read_to_string(&SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH).unwrap()
    )
        .and_then(|json| parse_btc_block_from_submission_material(&json))
}

pub fn get_sample_btc_tx() -> BtcTransaction {
    get_sample_testnet_block_and_txs()
        .unwrap()
        .block
        .txdata[SAMPLE_TRANSACTION_INDEX]
        .clone()
}

pub fn get_sample_op_return_btc_block_and_txs() -> BtcBlockAndId {
    parse_submission_material_to_json(
        &read_to_string(&SAMPLE_TESTNET_OP_RETURN_BTC_BLOCK_JSON).unwrap()

    )
        .and_then(|json| parse_btc_block_from_submission_material(&json))
        .unwrap()
}

pub fn get_sample_btc_op_return_tx() -> BtcTransaction {
    get_sample_op_return_btc_block_and_txs()
        .block
        .txdata[SAMPLE_OP_RETURN_TRANSACTION_INDEX]
        .clone()
}

pub fn get_sample_op_return_output() -> BtcTxOut {
    get_sample_btc_op_return_tx()
      .output[SAMPLE_OP_RETURN_TRANSACTION_OUTPUT_INDEX]
      .clone()
}

pub fn get_sample_btc_tx_output() -> BtcTxOut {
    get_sample_btc_tx()
        .output[SAMPLE_TRANSACTION_OUTPUT_INDEX]
        .clone()
}

pub fn get_sample_btc_utxo() -> BtcUtxo {
    let tx = get_sample_btc_tx();
    let outpoint = BtcOutPoint {
        txid: tx.txid(),
        vout: SAMPLE_TRANSACTION_OUTPUT_INDEX as u32,
    };
    BtcUtxo {
        witness: vec![], // NOTE: Array of byte arrays (empty for non-segwit).
        sequence: 4294967295, // NOTE: Unused so just "0xFFFFFFFF" hardcoded
        previous_output: outpoint,
        script_sig: get_sample_pay_to_pub_key_hash_script(),
    }
}

pub fn get_sample_op_return_utxo_and_value() -> BtcUtxoAndValue {
    create_op_return_btc_utxo_and_value_from_tx_output(
        &get_sample_btc_tx(),
        SAMPLE_OUTPUT_INDEX_OF_UTXO,
    )
}

pub fn get_sample_op_return_utxo_with_value_too_low() -> BtcUtxoAndValue {
    create_op_return_btc_utxo_and_value_from_tx_output(
        &get_sample_btc_block_n(9)
            .unwrap()
            .block
            .txdata
            [18],
        0,
    )
}

pub fn get_sample_p2sh_utxo_with_value_too_low() -> BtcUtxoAndValue {
    create_op_return_btc_utxo_and_value_from_tx_output(
        &get_sample_btc_block_n(9)
            .unwrap()
            .block
            .txdata
            [19],
        0,
    )
}

pub fn get_sample_utxo_and_values() -> Vec<BtcUtxoAndValue> {
    vec![
        get_sample_op_return_utxo_and_value_n(2).unwrap(),
        get_sample_op_return_utxo_and_value_n(3).unwrap(),
        get_sample_op_return_utxo_and_value_n(4).unwrap(),
        get_sample_op_return_utxo_and_value(),
        get_sample_op_return_utxo_with_value_too_low(),
        get_sample_p2sh_utxo_with_value_too_low(),
    ]
}

// TODO make fxn for getting these!
pub fn get_sample_p2sh_utxo_and_value() -> Result<BtcUtxoAndValue> {
    get_sample_btc_block_n(5)
        .map(|block_and_id| {
            let output_index = 0;
            let tx = block_and_id.block.txdata[1].clone();
            let nonce = 1337;
            let btc_deposit_address = "2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2"
                .to_string();
            let eth_address = "fedfe2616eb3661cb8fed2782f5f0cc91d59dcac"
                .to_string();
            let eth_address_and_nonce_hash =
            "98eaf3812c998a46e0ee997ccdadf736c7bc13c18a5292df7a8d39089fd28d9e"
                    .to_string();
            let deposit_info_json = DepositAddressInfoJson::new(
                nonce,
                eth_address,
                btc_deposit_address,
                eth_address_and_nonce_hash,
            );
            BtcUtxoAndValue::new(
                tx.output[output_index].value,
                &create_unsigned_utxo_from_tx(&tx, output_index as u32),
                Some(deposit_info_json),
                None,
            )
        })
}

pub fn get_sample_p2sh_utxo_and_value_2() -> Result<BtcUtxoAndValue> {
    get_sample_btc_block_n(10)
        .map(|block_and_id| {
            let output_index = 0;
            let tx = block_and_id.block.txdata[50].clone();
            let nonce = 1584612094;
            let btc_deposit_address = "2NB1SRPSujETy9zYbXRZAGkk1zuDZHFAtyk"
                .to_string();
            let eth_address = "provabletest"
                .to_string();
            let eth_address_and_nonce_hash =
            "1729dce0b4e54e39610a95376a8bc96335fd93da68ae6aa27a62d4c282fb1ad3"
                    .to_string();
            let deposit_info_json = DepositAddressInfoJson::new(
                nonce,
                eth_address,
                btc_deposit_address,
                eth_address_and_nonce_hash,
            );
            BtcUtxoAndValue::new(
                tx.output[output_index].value,
                &create_unsigned_utxo_from_tx(&tx, output_index as u32),
                Some(deposit_info_json),
                None,
            )
        })
}

pub fn get_sample_p2sh_utxo_and_value_3() -> Result<BtcUtxoAndValue> {
    get_sample_btc_block_n(11)
        .map(|block_and_id| {
            let output_index = 0;
            let tx = block_and_id.block.txdata[95].clone();
            let nonce = 1584696514;
            let btc_deposit_address = "2My5iu4S78DRiH9capTKo8sXEa98yHVkQXg"
                .to_string();
            let eth_address = "provabletest"
                .to_string();
            let eth_address_and_nonce_hash =
            "d11539e07a521c78c20381c98cc546e3ccdd8a5c97f1cffe83ae5537f61a6e39"
                .to_string();
            let deposit_info_json = DepositAddressInfoJson::new(
                nonce,
                eth_address,
                btc_deposit_address,
                eth_address_and_nonce_hash,
            );
            BtcUtxoAndValue::new(
                tx.output[output_index].value,
                &create_unsigned_utxo_from_tx(&tx, output_index as u32),
                Some(deposit_info_json),
                None,
            )
        })
}

pub fn get_sample_p2sh_utxo_and_value_4() -> Result<BtcUtxoAndValue> {
    get_sample_btc_block_n(12)
        .map(|block_and_id| {
            let output_index = 0;
            let tx = block_and_id.block.txdata[135].clone();
            let nonce = 1584696514;
            let btc_deposit_address = "2Mz5K485NS6V1yGKpUnjTrB8HJv7DKSpUgj"
                .to_string();
            let eth_address = "provabletest"
                .to_string();
            let eth_address_and_nonce_hash =
            "bf2221253331b654b56bceed49fbc9dd794dbefd1c9785c018143341ab13b312"
                .to_string();
            let deposit_info_json = DepositAddressInfoJson::new(
                nonce,
                eth_address,
                btc_deposit_address,
                eth_address_and_nonce_hash,
            );
            BtcUtxoAndValue::new(
                tx.output[output_index].value,
                &create_unsigned_utxo_from_tx(&tx, output_index as u32),
                Some(deposit_info_json),
                None,
            )
        })
}

pub fn get_sample_btc_block_n(n: usize) -> Result<BtcBlockAndId> {
    let block_path = match n {
        2 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_2),
        3 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_3),
        4 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_3),
        5 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_5),
        6 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_6),
        7 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_7),
        8 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_8),
        9 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_9),
        10 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_10),
        11 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_11),
        12 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_12),
        _ => Err(AppError::Custom(
            "✘ Don't have sample for that number!"
                .to_string()
        ))
    }.unwrap();
    parse_submission_material_to_json(&read_to_string(&block_path)?)
        .and_then(|json| parse_btc_block_from_submission_material(&json))
}

pub fn get_sample_op_return_utxo_and_value_n(n: usize) -> Result<BtcUtxoAndValue> {
    // NOTE: Tuple = path on disk, block_index of utxo & output_index of utxo!
    let tuple = match n {
        2 => Ok((SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_2, 18, 2)),
        3 => Ok((SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_3, 28, 0)),
        4 => Ok((SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_3, 28, 1)),
        _ => Err(AppError::Custom(
            "✘ Don't have sample for that number!"
                .to_string()
        ))
    }.unwrap();
    parse_submission_material_to_json(&read_to_string(&tuple.0)?)
        .and_then(|json| parse_btc_block_from_submission_material(&json))
        .map(|block_and_id| block_and_id.block.txdata[tuple.1].clone())
        .map(|tx| create_op_return_btc_utxo_and_value_from_tx_output(&tx, tuple.2))
}

pub fn get_sample_pay_to_pub_key_hash_script() -> BtcScript {
    get_pay_to_pub_key_hash_script(SAMPLE_TARGET_BTC_ADDRESS)
        .unwrap()
}

pub fn get_sample_btc_private_key() -> BtcPrivateKey {
    BtcPrivateKey::from_wif(SAMPLE_BTC_PRIVATE_KEY_WIF)
        .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_sample_sequential_block_and_ids() {
        get_sample_sequential_btc_blocks_in_db_format();
    }

    #[test]
    fn should_not_panic_getting_sample_btc_block_string() {
        get_sample_btc_block_json_string();
    }

    #[test]
    fn should_not_panic_getting_sample_btc_block_json() {
        get_sample_btc_block_json()
            .unwrap();
    }

    #[test]
    fn should_not_panic_getting_sample_btc_block() {
        get_sample_btc_block_and_id()
            .unwrap();
    }

    #[test]
    fn should_not_panic_getting_testnet_sample_block() {
        get_sample_testnet_block_and_txs()
            .unwrap();
    }
}
