use std::collections::HashMap;
use serde_json::{
    json,
    Value as JsonValue,
};
use ethereum_types::{
    H256,
    U256,
    Bloom,
    Address,
};
use crate::{
    btc_on_eth::eth::trie_nodes::Node,
    types::{
        Bytes,
        Result,
    },
    chains::eth::{
        any_sender::relay_transaction::RelayTransaction,
        eth_crypto::{
            eth_private_key::EthPrivateKey,
            eth_transaction::EthTransaction,
        },
    },
};

pub type EthHash = H256;
pub type EthAddress = Address;
pub type EthTopic = EthHash;
pub type NodeStack = Vec<Node>;
pub type EthSignature = [u8; 65];
pub type EthReceipts = Vec<EthReceipt>;
pub type EthSignedTransaction = String;
pub type ChildNodes = [Option<Bytes>; 16];
pub type TrieHashMap = HashMap<H256, Bytes>;
pub type EthTransactions = Vec<EthTransaction>;
pub type RelayTransactions = Vec<RelayTransaction>;

#[cfg(test)]
pub type EthTopics = Vec<EthTopic>;
#[cfg(test)]
pub type EthLogs = Vec<EthLog>;

#[derive(Debug)]
pub struct EthSigningParams {
    pub chain_id: u8,
    pub gas_price: u64,
    pub eth_account_nonce: u64,
    pub eth_private_key: EthPrivateKey,
    pub ptoken_contract_address: EthAddress,
}

#[derive(Debug)]
pub struct AnySenderSigningParams {
    pub chain_id: u8,
    pub any_sender_nonce: u64,
    pub eth_private_key: EthPrivateKey,
    pub public_eth_address: EthAddress,
    pub erc777_proxy_address: EthAddress,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RedeemParams {
    pub amount: U256,
    pub from: EthAddress,
    pub recipient: String,
    pub originating_tx_hash: EthHash,
}

impl RedeemParams {
    pub fn new(
        amount: U256,
        from: EthAddress,
        recipient: String,
        originating_tx_hash: EthHash,
    ) -> RedeemParams {
        RedeemParams { amount, recipient, originating_tx_hash, from }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EthBlockAndReceipts {
    pub block: EthBlock,
    pub receipts: Vec<EthReceipt>
}

#[derive(Clone, Debug, Deserialize)]
pub struct EthBlockAndReceiptsJson {
    pub block: EthBlockJson,
    pub receipts: Vec<EthReceiptJson>
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EthReceipt {
    pub to: Address,
    pub from: Address,
    pub status: bool,
    pub gas_used: U256,
    pub block_hash: H256,
    pub transaction_hash: H256,
    pub cumulative_gas_used: U256,
    pub block_number: U256,
    pub transaction_index: U256,
    pub contract_address: Address,
    pub logs: Vec<EthLog>,
    pub logs_bloom: Bloom,
}

impl EthReceipt {
    pub fn to_json(&self) -> Result<JsonValue> {
        let encoded_logs = self
            .logs
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
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EthBlock {
    pub difficulty: U256,
    pub extra_data: Bytes,
    pub gas_limit: U256,
    pub gas_used: U256,
    pub hash: H256,
    pub logs_bloom: Bloom,
    pub miner: Address,
    pub mix_hash: H256,
    pub nonce: Bytes,
    pub number: U256,
    pub parent_hash: H256,
    pub receipts_root: H256,
    pub sha3_uncles: H256,
    pub size: U256,
    pub state_root: H256,
    pub timestamp: U256,
    pub total_difficulty: U256,
    pub transactions: Vec<H256>,
    pub transactions_root: H256,
    pub uncles: Vec<H256>,
}

#[allow(non_snake_case)]
#[derive(Clone, Debug, Deserialize)]
pub struct EthBlockJson {
    pub difficulty: String,
    pub extraData: String,
    pub gasLimit: usize,
    pub gasUsed: usize,
    pub hash: String,
    pub logsBloom: String,
    pub miner: String,
    pub mixHash: String,
    pub nonce: String,
    pub number: usize,
    pub parentHash: String,
    pub receiptsRoot: String,
    pub sha3Uncles: String,
    pub size: usize,
    pub stateRoot: String,
    pub timestamp: usize,
    pub totalDifficulty: String,
    pub transactions: Vec<String>,
    pub transactionsRoot: String,
    pub uncles: Vec<String>,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::eth_test_utils::{
        get_sample_log_with_desired_topic,
        get_sample_receipt_with_desired_topic,
    };

    #[test]
    fn should_encode_eth_log_as_json() {
        let log = get_sample_log_with_desired_topic();
        let result = log.to_json().unwrap();
        let expected_result = json!({
            "address": "0x60a640e2d10e020fee94217707bfa9543c8b59e0",
            "data": "0x00000000000000000000000000000000000000000000000589ba7ab174d54000",
            "topics": vec![
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                "0x000000000000000000000000250abfa8bc8371709fa4b601d821b1421667a886",
                "0x0000000000000000000000005a7dd68907e103c3239411dae0b0eef968468ef2",
            ]
        });
        assert_eq!(result, expected_result);
    }

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
}
