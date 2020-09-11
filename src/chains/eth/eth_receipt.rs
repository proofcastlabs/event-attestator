use rlp::{
    RlpStream,
    Encodable,
};
use serde_json::{
    json,
    Value as JsonValue,
};
use ethereum_types::{
    H160,
    U256,
    Bloom,
    H256 as EthHash,
    Address as EthAddress,
};
use crate::{

    types::{
        Bytes,
        Result,
    },
    btc_on_eth::{
        utils::{
            convert_hex_to_h256,
            convert_hex_to_address,
            convert_json_value_to_string,
        },
        eth::{
            trie::{
                Trie,
                put_in_trie_recursively,
            },
            nibble_utils::{
                Nibbles,
                get_nibbles_from_bytes,
            },
        },
    },
    chains::eth::eth_log::{
        EthLogs,
        EthLogJson,
    },
};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct EthReceipts(pub Vec<EthReceipt>);

impl EthReceipts {
    pub fn new(receipts: Vec<EthReceipt>) -> Self {
        Self(receipts)
    }

    pub fn new_empty() -> Self {
        Self(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn from_jsons(jsons: &[EthReceiptJson]) -> Result<Self> {
        Ok(Self(jsons.iter().cloned().map(|json| EthReceipt::from_json(&json)).collect::<Result<Vec<EthReceipt>>>()?))
    }

    fn filter_for_receipts_containing_log_with_address(&self, address: &EthAddress) -> Self {
        Self::new(self.0.iter().filter(|receipt| receipt.contains_log_with_address(address)).cloned().collect())
    }

    fn filter_for_receipts_containing_log_with_topic(&self, topic: &EthHash) -> Self {
        Self::new(self.0.iter().filter(|receipt| receipt.contains_log_with_topic(topic)).cloned().collect())
    }

    fn filter_for_receipts_containing_log_with_address_and_topic(&self, address: &EthAddress, topic: &EthHash) -> Self {
        self
            .filter_for_receipts_containing_log_with_address(address)
            .filter_for_receipts_containing_log_with_topic(topic)
    }

    pub fn filter_for_receipts_containing_log_with_address_and_topics(
        &self,
        address: &EthAddress,
        topics: &[EthHash],
    ) -> Self {
        Self::new(
            topics
                .iter()
                .map(|topic| self.filter_for_receipts_containing_log_with_address_and_topic(address, topic).0)
                .flatten()
                .collect()
        )
    }

    pub fn get_rlp_encoded_receipts_and_nibble_tuples(&self) -> Result<Vec<(Nibbles, Bytes)>> {
        self.0.iter().map(|receipt| receipt.get_rlp_encoded_receipt_and_encoded_key_tuple()).collect()
    }

    pub fn get_merkle_root(&self) -> Result<EthHash> {
        self
            .get_rlp_encoded_receipts_and_nibble_tuples()
            .and_then(|key_value_tuples| put_in_trie_recursively(Trie::get_new_trie()?, key_value_tuples, 0))
            .map(|trie| trie.root)
    }
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Deserialize)]
pub struct EthReceiptJson {
    pub from: String,
    pub status: bool,
    pub gasUsed: usize,
    pub blockHash: String,
    pub logsBloom: String,
    pub logs: Vec<EthLogJson>,
    pub blockNumber: usize,
    pub to: serde_json::Value,
    pub transactionHash: String,
    pub transactionIndex: usize,
    pub cumulativeGasUsed: usize,
    pub contractAddress: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EthReceipt {
    pub to: EthAddress,
    pub from: EthAddress,
    pub status: bool,
    pub gas_used: U256,
    pub block_hash: EthHash,
    pub transaction_hash: EthHash,
    pub cumulative_gas_used: U256,
    pub block_number: U256,
    pub transaction_index: U256,
    pub contract_address: EthAddress,
    pub logs: EthLogs,
    pub logs_bloom: Bloom,
}

impl EthReceipt {
    pub fn to_json(&self) -> Result<JsonValue> {
        let encoded_logs = self
            .logs
            .0
            .iter()
            .map(|eth_log| eth_log.to_json())
            .collect::<Result<Vec<JsonValue>>>()?;
        Ok(
            json!({
                "logs": encoded_logs,
                "status": self.status,
                "gasUsed": self.gas_used.as_usize(),
                "blockNumber": self.block_number.as_usize(),
                "transactionIndex": self.transaction_index.as_usize(),
                "to": format!("0x{}", hex::encode(self.to.as_bytes())),
                "cumulativeGasUsed": self.cumulative_gas_used.as_usize(),
                "from": format!("0x{}", hex::encode(self.from.as_bytes())),
                "contractAddress": format!("0x{:x}", self.contract_address),
                "blockHash": format!("0x{}", hex::encode(self.block_hash.as_bytes())),
                "logsBloom": format!("0x{}", hex::encode(self.logs_bloom.as_bytes())),
                "transactionHash": format!("0x{}", hex::encode( self.transaction_hash.as_bytes())),
            })
        )
    }

    pub fn from_json(eth_receipt_json: &EthReceiptJson) -> Result<Self> {
        let logs = EthLogs::from_receipt_json(&eth_receipt_json)?;
        Ok(
            EthReceipt {
                status: eth_receipt_json.status,
                logs_bloom: logs.get_bloom(),
                gas_used: U256::from(eth_receipt_json.gasUsed),
                from: convert_hex_to_address(&eth_receipt_json.from)?,
                block_number: U256::from(eth_receipt_json.blockNumber),
                block_hash: convert_hex_to_h256(&eth_receipt_json.blockHash)?,
                transaction_index: U256::from(eth_receipt_json.transactionIndex),
                cumulative_gas_used: U256::from(eth_receipt_json.cumulativeGasUsed),
                transaction_hash: convert_hex_to_h256(&eth_receipt_json.transactionHash)?,
                to: match eth_receipt_json.to {
                    serde_json::Value::Null => H160::zero(),
                    _ => convert_hex_to_address(&convert_json_value_to_string(&eth_receipt_json.to)?)?,
                },
                contract_address: match eth_receipt_json.contractAddress {
                    serde_json::Value::Null => EthAddress::zero(),
                    _ => convert_hex_to_address(&convert_json_value_to_string(&eth_receipt_json.contractAddress)?)?,
                },
                logs,
            }
        )
    }

    pub fn contains_log_with_topic(&self, topic: &EthHash) -> bool {
        self.logs.contain_topic(topic)
    }

    pub fn contains_log_with_address(&self, address: &EthAddress) -> bool {
        self.logs.contain_address(address)
    }

    pub fn rlp_encode(&self) -> Result<Bytes> {
        let mut rlp_stream = RlpStream::new();
        rlp_stream.append(self);
        Ok(rlp_stream.out())
    }

    fn rlp_encode_transaction_index(&self) -> Bytes {
        let mut rlp_stream = RlpStream::new();
        rlp_stream.append(&self.transaction_index.as_usize());
        rlp_stream.out()
    }

    pub fn get_rlp_encoded_receipt_and_encoded_key_tuple(&self) -> Result<(Nibbles, Bytes)> {
        self
            .rlp_encode()
            .and_then(|bytes| Ok((get_nibbles_from_bytes(self.rlp_encode_transaction_index()), bytes)))
    }
}

impl Encodable for EthReceipt {
    fn rlp_append(&self, rlp_stream: &mut RlpStream) {
        let rlp = rlp_stream.begin_list(4);
        match &self.status {
            true => rlp.append(&self.status),
            false => rlp.append_empty_data()
        };
        rlp.append(&self.cumulative_gas_used).append(&self.logs_bloom).append_list(&self.logs.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::eth_test_utils::{
        get_expected_receipt,
        SAMPLE_RECEIPT_INDEX,
        get_sample_contract_topic,
        get_sample_contract_address,
        get_sample_eth_block_and_receipts,
        get_sample_receipt_with_desired_topic,
        get_sample_eth_block_and_receipts_json,
        get_valid_state_with_invalid_block_and_receipts,
    };

    #[test]
    fn should_encode_eth_receipt_as_json() {
        let receipt = get_sample_receipt_with_desired_topic();
        let result = receipt.to_json().unwrap();
        let expected_result = json!({
            "status": true,
            "gasUsed": 37947,
            "transactionIndex": 2,
            "blockNumber": 8503804,
            "cumulativeGasUsed": 79947,
            "to": "0x60a640e2d10e020fee94217707bfa9543c8b59e0",
            "from": "0x250abfa8bc8371709fa4b601d821b1421667a886",
            "contractAddress": "0x0000000000000000000000000000000000000000",
            "blockHash": "0xb626a7546311dd56c6f5e9fd07d00c86074077bbd6d5a4c4f8269a2490aa47c0",
            "transactionHash":  "0xab8078c9aa8720c5f9206bd2673f25f359d8a01b62212da99ff3b53c1ca3d440",
            "logsBloom": "0x00000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000010000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000800000000000000000000010000000000000000008000000000000000000000000000000000000000000000200000003000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000020000000",
            "logs": vec![
                json!({
                    "address": "0x60a640e2d10e020fee94217707bfa9543c8b59e0",
                    "data": "0x00000000000000000000000000000000000000000000000589ba7ab174d54000",
                    "topics": vec![
                        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                        "0x000000000000000000000000250abfa8bc8371709fa4b601d821b1421667a886",
                        "0x0000000000000000000000005a7dd68907e103c3239411dae0b0eef968468ef2",
                    ],
                })
            ],
        });
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_encode_eth_block_and_receipts_as_json() {
        let block_and_receipts = get_sample_eth_block_and_receipts();
        let result = block_and_receipts.to_json();
        assert!(result.is_ok());
    }

    #[test]
    fn should_encode_eth_block_and_receipts_as_bytes() {
        let block_and_receipts = get_sample_eth_block_and_receipts();
        let result = block_and_receipts.to_bytes();
        assert!(result.is_ok());
    }

    #[test]
    fn should_parse_eth_receipt_json() {
        let eth_json = get_sample_eth_block_and_receipts_json().unwrap();
        let receipt_json = eth_json.receipts[SAMPLE_RECEIPT_INDEX].clone();
        match EthReceipt::from_json(&receipt_json) {
            Ok(receipt) => assert_eq!(receipt, get_expected_receipt()),
            _ => panic!("Should have parsed receipt!"),
        }
    }

    #[test]
    fn should_parse_eth_receipt_jsons() {
        let eth_json = get_sample_eth_block_and_receipts_json().unwrap();
        if EthReceipts::from_jsons(&eth_json.receipts).is_err() {
            panic!("Should have generated receipts correctly!")
        }
    }

    #[test]
    fn should_filter_receipts_for_topics() {
        let expected_num_receipts_after = 1;
        let receipts = get_sample_eth_block_and_receipts().receipts;
        let num_receipts_before = receipts.len();
        let topic = get_sample_contract_topic();
        let topics = vec![topic];
        let address = get_sample_contract_address();
        let result = receipts.filter_for_receipts_containing_log_with_address_and_topics(&address, &topics);
        let num_receipts_after = result.len();
        assert_eq!(num_receipts_after, expected_num_receipts_after);
        assert!(num_receipts_before > num_receipts_after);
        result.0.iter().map(|receipt| assert!(receipt.logs.contain_topic(&topic))).for_each(drop);
    }

    fn get_encoded_receipt() -> String {
        "f901a7018301384bb9010000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000010000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000800000000000000000000010000000000000000008000000000000000000000000000000000000000000000200000003000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000020000000f89df89b9460a640e2d10e020fee94217707bfa9543c8b59e0f863a0ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3efa0000000000000000000000000250abfa8bc8371709fa4b601d821b1421667a886a00000000000000000000000005a7dd68907e103c3239411dae0b0eef968468ef2a000000000000000000000000000000000000000000000000589ba7ab174d54000".to_string()
    }

    #[test]
    fn should_rlp_encode_receipt() {
        let result =get_expected_receipt().rlp_encode().unwrap();
        assert_eq!(hex::encode(result), get_encoded_receipt())
    }

    #[test]
    fn should_get_encoded_receipt_and_hash_tuple() {
        let result = get_expected_receipt().get_rlp_encoded_receipt_and_encoded_key_tuple().unwrap();
        let expected_encoded_nibbles = get_nibbles_from_bytes(vec![0x02]); // NOTE: The tx index of sample receipt
        assert_eq!(result.0, expected_encoded_nibbles);
        assert_eq!(hex::encode(result.1), get_encoded_receipt());
    }

    #[test]
    fn should_get_encoded_receipts_and_hash_tuples() {
        let expected_encoded_nibbles = get_nibbles_from_bytes(vec![0x02]);
        let receipts = EthReceipts::new(vec![get_expected_receipt(), get_expected_receipt()]);
        let results = receipts.get_rlp_encoded_receipts_and_nibble_tuples().unwrap();
        results
            .iter()
            .map(|result| {
                assert_eq!(result.0, expected_encoded_nibbles);
                assert_eq!(hex::encode(&result.1), get_encoded_receipt());
            })
            .for_each(drop);
    }

    #[test]
    fn should_get_receipts_merkle_root_from_receipts() {
        let block_and_receipts = get_sample_eth_block_and_receipts();
        let result = block_and_receipts.receipts.get_merkle_root().unwrap();
        let expected_result = block_and_receipts.block.receipts_root;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_return_false_if_receipts_root_is_not_correct() {
        let state = get_valid_state_with_invalid_block_and_receipts().unwrap();
        let block_and_receipts = state.get_eth_block_and_receipts().unwrap();
        let result = block_and_receipts.receipts_are_valid().unwrap();
        assert!(!result);
    }
}
