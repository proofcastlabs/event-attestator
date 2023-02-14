use std::{fs::read_to_string, str::FromStr};

use bitcoin::blockdata::transaction::Transaction as BtcTransaction;
use common::{errors::AppError, types::Result};
use common_btc::{
    create_unsigned_utxo_from_tx,
    BtcBlockAndId,
    BtcPrivateKey,
    BtcSubmissionMaterialJson,
    BtcUtxoAndValue,
    BtcUtxosAndValues,
};

pub const SAMPLE_TRANSACTION_INDEX: usize = 1;
pub const SAMPLE_OUTPUT_INDEX_OF_UTXO: u32 = 0;
pub const SAMPLE_BTC_PRIVATE_KEY_WIF: &str = "cP2Dv4mx1DwJzN8iF6CCyPZmuS27bT9MV4Qmgb9h6cNQNq2Jgpmy";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH: &str =
    "src/test_utils/1610046-testnet-block-with-tx-to-test-address.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_2: &str =
    "src/test_utils/1610166-testnet-block-with-tx-to-test-address.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_3: &str =
    "src/test_utils/1610161-testnet-block-with-tx-to-test-address.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_5: &str =
    "src/test_utils/1637173-testnet-block-and-txs-with-p2sh-deposit.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_6: &str =
    "src/test_utils/1660807-testnet-block-and-txs-with-2-p2sh-deposits.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_7: &str =
    "src/test_utils/btc-1661479-btc-block-and-txs-with-deposit-originating-from-enclave-key.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_8: &str = "src/test_utils/1661611-block-and-txs-with-no-op-return.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_9: &str =
    "src/test_utils/1666951-block-and-txs-p2sh-and-op-return-below-threshold.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_10: &str = "src/test_utils/btc-1670411-block-and-txs.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_11: &str = "src/test_utils/btc-1670534-block-and-txs.json";

pub const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_12: &str = "src/test_utils/btc-1670541-block-and-txs.json";

const SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_13: &str = "src/test_utils/btc-761249-block-and-txs.json";

pub fn create_p2pkh_btc_utxo_and_value_from_tx_output(tx: &BtcTransaction, output_index: u32) -> BtcUtxoAndValue {
    BtcUtxoAndValue::new(
        tx.output[output_index as usize].value,
        &create_unsigned_utxo_from_tx(tx, output_index),
        None,
        None,
    )
}

pub fn get_sample_testnet_block_and_txs() -> Result<BtcBlockAndId> {
    BtcSubmissionMaterialJson::from_str(&read_to_string(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH).unwrap())
        .and_then(|json| BtcBlockAndId::from_json(&json))
}

pub fn get_sample_btc_tx() -> BtcTransaction {
    get_sample_testnet_block_and_txs().unwrap().block.txdata[SAMPLE_TRANSACTION_INDEX].clone()
}
pub fn get_sample_p2pkh_utxo_and_value() -> BtcUtxoAndValue {
    create_p2pkh_btc_utxo_and_value_from_tx_output(&get_sample_btc_tx(), SAMPLE_OUTPUT_INDEX_OF_UTXO)
}

pub fn get_sample_p2pkh_utxo_with_value_too_low() -> BtcUtxoAndValue {
    create_p2pkh_btc_utxo_and_value_from_tx_output(&get_sample_btc_block_n(9).unwrap().block.txdata[18], 0)
}

pub fn get_sample_p2sh_utxo_with_value_too_low() -> BtcUtxoAndValue {
    create_p2pkh_btc_utxo_and_value_from_tx_output(&get_sample_btc_block_n(9).unwrap().block.txdata[19], 0)
}

pub fn get_sample_utxo_and_values() -> BtcUtxosAndValues {
    BtcUtxosAndValues::new(vec![
        get_sample_p2pkh_utxo_and_value_n(2).unwrap(),
        get_sample_p2pkh_utxo_and_value_n(3).unwrap(),
        get_sample_p2pkh_utxo_and_value_n(4).unwrap(),
        get_sample_p2pkh_utxo_and_value(),
        get_sample_p2pkh_utxo_with_value_too_low(),
        get_sample_p2sh_utxo_with_value_too_low(),
    ])
}

pub fn get_sample_btc_block_n(n: usize) -> Result<BtcBlockAndId> {
    let block_path = match n {
        5 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_5),
        6 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_6),
        7 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_7),
        8 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_8),
        9 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_9),
        10 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_10),
        11 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_11),
        12 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_12),
        13 => Ok(SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_13),
        _ => Err(AppError::Custom("✘ Don't have sample for that number!".into())),
    }
    .unwrap();
    BtcSubmissionMaterialJson::from_str(&read_to_string(block_path)?).and_then(|json| BtcBlockAndId::from_json(&json))
}

pub fn get_sample_p2pkh_utxo_and_value_n(n: usize) -> Result<BtcUtxoAndValue> {
    // NOTE: Tuple = path on disk, block_index of utxo & output_index of utxo!
    let tuple = match n {
        2 => Ok((SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_2, 18, 2)),
        3 => Ok((SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_3, 28, 0)),
        4 => Ok((SAMPLE_TESTNET_BTC_BLOCK_JSON_PATH_3, 28, 1)),
        _ => Err(AppError::Custom("✘ Don't have sample for that number!".into())),
    }
    .unwrap();
    BtcSubmissionMaterialJson::from_str(&read_to_string(tuple.0)?)
        .and_then(|json| BtcBlockAndId::from_json(&json))
        .map(|block_and_id| block_and_id.block.txdata[tuple.1].clone())
        .map(|tx| create_p2pkh_btc_utxo_and_value_from_tx_output(&tx, tuple.2))
}

pub fn get_sample_btc_private_key() -> BtcPrivateKey {
    BtcPrivateKey::from_wif(SAMPLE_BTC_PRIVATE_KEY_WIF).unwrap()
}
