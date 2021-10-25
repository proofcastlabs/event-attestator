use std::cmp::Ordering;

use derive_more::{Constructor, Deref, From, Into};
use ethereum_types::{Address as EthAddress, Bloom, H160, H256 as EthHash, U256};
use rlp::RlpStream;
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

use crate::{
    chains::{
        eth::eth_utils::{convert_hex_to_eth_address, convert_hex_to_h256, convert_json_value_to_string},
        evm::eth_log::{EthLog, EthLogJson, EthLogs},
    },
    types::{Bytes, Result},
};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Constructor, Deref, From, Into)]
pub struct EthReceipts(pub Vec<EthReceipt>);

impl EthReceipts {
    pub fn from_jsons(jsons: &[EthReceiptJson]) -> Result<Self> {
        Ok(Self(
            jsons
                .iter()
                .cloned()
                .map(|json| EthReceipt::from_json(&json))
                .collect::<Result<Vec<EthReceipt>>>()?,
        ))
    }

    fn get_receipts_containing_log_from_address(&self, address: &EthAddress) -> Self {
        Self::new(
            self.0
                .iter()
                .filter(|receipt| receipt.contains_log_from_address(address))
                .cloned()
                .collect(),
        )
    }

    fn get_receipts_containing_log_with_topic(&self, topic: &EthHash) -> Self {
        Self::new(
            self.0
                .iter()
                .filter(|receipt| receipt.contains_log_with_topic(topic))
                .cloned()
                .collect(),
        )
    }

    pub fn get_receipts_containing_logs_from_address_and_with_topic(
        &self,
        address: &EthAddress,
        topic: &EthHash,
    ) -> Self {
        self.get_receipts_containing_log_from_address(address)
            .get_receipts_containing_log_with_topic(topic)
    }

    pub fn get_receipts_containing_log_from_address_and_with_topics(
        &self,
        address: &EthAddress,
        topics: &[EthHash],
    ) -> Self {
        Self::new(
            topics
                .iter()
                .map(|topic| {
                    self.get_receipts_containing_logs_from_address_and_with_topic(address, topic)
                        .0
                })
                .collect::<Vec<Vec<EthReceipt>>>()
                .concat(),
        )
    }

    pub fn get_receipts_containing_log_from_addresses_and_with_topics(
        &self,
        addresses: &[EthAddress],
        topics: &[EthHash],
    ) -> Self {
        Self::new(
            addresses
                .iter()
                .map(|address| {
                    self.get_receipts_containing_log_from_address_and_with_topics(address, topics)
                        .0
                })
                .collect::<Vec<Vec<EthReceipt>>>()
                .concat(),
        )
    }

    fn get_logs(&self) -> EthLogs {
        EthLogs::new(
            self.iter()
                .cloned()
                .map(|receipt| receipt.logs.0)
                .collect::<Vec<Vec<EthLog>>>()
                .concat(),
        )
    }

    pub fn get_logs_from_address_with_topic(&self, address: &EthAddress, topic: &EthHash) -> EthLogs {
        self.get_logs()
            .filter_for_those_from_address_containing_topic(address, topic)
    }

    pub fn get_merkle_root(&self) -> Result<EthHash> {
        Err("No longer implemented!".into())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthReceiptJson {
    pub from: String,
    pub status: bool,
    pub gas_used: usize,
    pub block_hash: String,
    pub logs_bloom: String,
    pub logs: Vec<EthLogJson>,
    pub block_number: usize,
    pub to: serde_json::Value,
    pub transaction_hash: String,
    pub transaction_index: usize,
    pub cumulative_gas_used: usize,
    pub contract_address: serde_json::Value,
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
        Ok(json!({
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
        }))
    }

    pub fn from_json(eth_receipt_json: &EthReceiptJson) -> Result<Self> {
        let logs = EthLogs::from_receipt_json(eth_receipt_json)?;
        Ok(EthReceipt {
            status: eth_receipt_json.status,
            logs_bloom: logs.get_bloom(),
            gas_used: U256::from(eth_receipt_json.gas_used),
            from: convert_hex_to_eth_address(&eth_receipt_json.from)?,
            block_number: U256::from(eth_receipt_json.block_number),
            block_hash: convert_hex_to_h256(&eth_receipt_json.block_hash)?,
            transaction_index: U256::from(eth_receipt_json.transaction_index),
            cumulative_gas_used: U256::from(eth_receipt_json.cumulative_gas_used),
            transaction_hash: convert_hex_to_h256(&eth_receipt_json.transaction_hash)?,
            to: match eth_receipt_json.to {
                serde_json::Value::Null => H160::zero(),
                _ => convert_hex_to_eth_address(&convert_json_value_to_string(&eth_receipt_json.to)?)?,
            },
            contract_address: match eth_receipt_json.contract_address {
                serde_json::Value::Null => EthAddress::zero(),
                _ => convert_hex_to_eth_address(&convert_json_value_to_string(&eth_receipt_json.contract_address)?)?,
            },
            logs,
        })
    }

    pub fn contains_log_with_topic(&self, topic: &EthHash) -> bool {
        self.logs.contain_topic(topic)
    }

    pub fn contains_log_from_address(&self, address: &EthAddress) -> bool {
        self.logs.contain_address(address)
    }

    pub fn rlp_encode(&self) -> Result<Bytes> {
        let mut rlp_stream = RlpStream::new();
        rlp_stream.begin_list(4);
        match &self.status {
            true => rlp_stream.append(&self.status),
            false => rlp_stream.append_empty_data(),
        };
        rlp_stream
            .append(&self.cumulative_gas_used)
            .append(&self.logs_bloom)
            .append_list(&self.logs);
        Ok(rlp_stream.out().to_vec())
    }

    fn rlp_encode_transaction_index(&self) -> Bytes {
        let mut rlp_stream = RlpStream::new();
        rlp_stream.append(&self.transaction_index.as_usize());
        rlp_stream.out().to_vec()
    }

    pub fn get_logs_from_address_with_topic(&self, address: &EthAddress, topic: &EthHash) -> EthLogs {
        EthLogs::new(
            self.logs
                .iter()
                .filter(|log| log.is_from_address(address) && log.contains_topic(topic))
                .cloned()
                .collect(),
        )
    }

    pub fn get_logs_from_addresses_with_topic(&self, addresses: &[EthAddress], topic: &EthHash) -> EthLogs {
        EthLogs::new(
            addresses
                .iter()
                .map(|address| self.get_logs_from_address_with_topic(address, topic).0)
                .collect::<Vec<Vec<EthLog>>>()
                .concat(),
        )
    }
}

impl Ord for EthReceipt {
    fn cmp(&self, other: &Self) -> Ordering {
        self.transaction_index.cmp(&other.transaction_index)
    }
}

impl PartialOrd for EthReceipt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
