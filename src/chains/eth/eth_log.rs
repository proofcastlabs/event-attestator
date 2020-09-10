use serde_json::{
    json,
    Value as JsonValue,
};
use ethereum_types::{
    H256,
    Bloom,
    Address,
    BloomInput,
};
use crate::{
    chains::eth::eth_receipt::EthReceiptJson,
    types::{
        Bytes,
        Result,
    },
    btc_on_eth::utils::{
        convert_hex_to_bytes,
        convert_hex_to_address,
        convert_hex_strings_to_h256s,
    },
};

#[allow(non_snake_case)]
#[derive(Clone, Debug, Deserialize)]
pub struct EthLogJson {
    pub data: String,
    pub address: String,
    pub topics: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EthLog {
    pub address: Address,
    pub topics: Vec<H256>,
    pub data: Bytes,
}

impl EthLog {
    pub fn from_json(log_json: &EthLogJson) -> Result<Self> {
        Ok(
            EthLog {
                data: convert_hex_to_bytes(&log_json.data)?,
                address: convert_hex_to_address(&log_json.address)?,
                topics: convert_hex_strings_to_h256s(log_json.topics.iter().map(AsRef::as_ref).collect())?,
            }
        )
    }

    pub fn to_json(&self) -> Result<JsonValue> {
        let topic_strings = self
            .topics
            .iter()
            .map(|topic_hash| format!("0x{}", hex::encode(topic_hash.as_bytes())))
            .collect::<Vec<String>>();
        Ok(
            json!({
                "topics": topic_strings,
                "data": format!("0x{}", hex::encode(self.data.clone())),
                "address": format!("0x{}", hex::encode(self.address.as_bytes())),
            })
        )
    }

    pub fn get_bloom(&self) -> Bloom {
        self
            .topics
            .iter()
            .fold(
                Bloom::from(BloomInput::Raw(self.address.as_bytes())),
                |mut bloom, topic| {
                    bloom.accrue(BloomInput::Raw(topic.as_bytes()));
                    bloom
                }
            )
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct EthLogs(pub Vec<EthLog>);

impl EthLogs {
    pub fn get_bloom(&self) -> Bloom {
        self
            .0
            .iter()
            .fold(Bloom::default(), |mut bloom, log| {
                bloom.accrue_bloom(&log.get_bloom());
                bloom
            })
    }

    pub fn from_receipt_json(json: &EthReceiptJson) -> Result<Self> {
        Ok(Self(json.logs.iter().map(|log_json| EthLog::from_json(log_json)).collect::<Result<Vec<EthLog>>>()?))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::eth_test_utils::{
        get_expected_log,
        SAMPLE_RECEIPT_INDEX,
        get_sample_eth_block_and_receipts_json,
    };

    #[test]
    fn should_get_logs_bloom_from_logs() {
        let expected_bloom = "00000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000010000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000800000000000000000000010000000000000000008000000000000000000000000000000000000000000000200000003000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000020000000";
        let expected_bloom_bytes = &hex::decode(expected_bloom)
            .unwrap()[..];
        let eth_block_and_receipt_json = get_sample_eth_block_and_receipts_json()
            .unwrap();
        let receipt_json = eth_block_and_receipt_json
            .receipts[SAMPLE_RECEIPT_INDEX]
            .clone();
        let logs = EthLogs::from_receipt_json(&receipt_json).unwrap();
        let result = logs.get_bloom();
        assert_eq!(result.as_bytes(), expected_bloom_bytes)
    }

    #[test]
    fn should_get_logs_from_receipt_json() {
        let expected_result = get_expected_log();
        let eth_block_and_receipt_json = get_sample_eth_block_and_receipts_json().unwrap();
        let result = EthLogs::from_receipt_json(&eth_block_and_receipt_json.receipts[SAMPLE_RECEIPT_INDEX])
            .unwrap();
        assert_eq!(result.0[0], expected_result);
    }

    #[test]
    fn should_get_log_from_log_json_correctly() {
        let eth_block_and_receipt_json = get_sample_eth_block_and_receipts_json()
            .unwrap();
        let log_json = eth_block_and_receipt_json
            .receipts[SAMPLE_RECEIPT_INDEX]
            .logs[0]
            .clone();
        let result = EthLog::from_json(&log_json).unwrap();
        let expected_result = get_expected_log();
        assert_eq!(result, expected_result)
    }

}
