use std::collections::HashMap;
use serde_json::{
    json,
    Value as JsonValue,
};
use ethereum_types::{
    H256,
    U256,
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
        eth_block::{
            EthBlock,
            EthBlockJson,
        },
        eth_receipt::{
            EthReceipt,
            EthReceiptJson,
        },
        eth_crypto::{
            eth_private_key::EthPrivateKey,
            eth_transaction::EthTransaction,
        },
    },
};

pub type EthHash = H256;
pub type EthTopic = EthHash;
pub type EthAddress = Address;
pub type NodeStack = Vec<Node>;
pub type EthSignature = [u8; 65];
pub type EthSignedTransaction = String;
pub type ChildNodes = [Option<Bytes>; 16];
pub type TrieHashMap = HashMap<H256, Bytes>;
pub type EthTransactions = Vec<EthTransaction>;
pub type RelayTransactions = Vec<RelayTransaction>;

#[cfg(test)]
pub type EthTopics = Vec<EthTopic>;

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
    pub fn new(amount: U256, from: EthAddress, recipient: String, originating_tx_hash: EthHash) -> RedeemParams {
        RedeemParams { amount, recipient, originating_tx_hash, from }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EthBlockAndReceipts {
    pub block: EthBlock,
    pub receipts: Vec<EthReceipt>
}

impl EthBlockAndReceipts {
    pub fn to_json(&self) -> Result<JsonValue> {
        Ok(json!({
            "block": &self.block.to_json()?,
            "receipts": self.receipts.iter().map(|receipt| receipt.to_json()).collect::<Result<Vec<JsonValue>>>()?,
        }))
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.to_json()?)?)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct EthBlockAndReceiptsJson {
    pub block: EthBlockJson,
    pub receipts: Vec<EthReceiptJson>
}
