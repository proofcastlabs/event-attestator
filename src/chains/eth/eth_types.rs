use std::collections::HashMap;
use serde_json::{
    json,
    Value as JsonValue,
};
use ethereum_types::{
    H256,
    H160,
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
    btc_on_eth::utils::{
        convert_hex_to_h256,
        convert_hex_to_address,
        convert_json_value_to_string,
    },
    chains::eth::{
        any_sender::relay_transaction::RelayTransaction,
        eth_log::{
            EthLogs,
            EthLogJson,
        },
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
                    serde_json::Value::Null => Address::zero(),
                    _ => convert_hex_to_address(&convert_json_value_to_string(&eth_receipt_json.contractAddress)?)?,
                },
                logs,
            }
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

impl EthBlock {
    pub fn to_json(&self) -> Result<JsonValue> {
        let encoded_transactions = self
            .transactions
            .iter()
            .map(|tx_hash| format!("0x{}", hex::encode(tx_hash.as_bytes())))
            .collect::<Vec<String>>();
        let encoded_uncles = self
            .uncles
            .iter()
            .map(|uncle_hash| format!("0x{}", hex::encode(uncle_hash.as_bytes())))
            .collect::<Vec<String>>();
        Ok(
            json!({
                "nonce": format!("0x{}", hex::encode(self.nonce.clone())),
                "uncles": encoded_uncles,
                "size": self.size.as_usize(),
                "number": self.number.as_usize(),
                "gasUsed": self.gas_used.as_usize(),
                "transactions": encoded_transactions,
                "gasLimit": self.gas_limit.as_usize(),
                "timestamp": self.timestamp.as_usize(),
                "difficulty": self.difficulty.to_string(),
                "totalDifficulty": self.total_difficulty.to_string(),
                "logsBloom": format!("0x{}", hex::encode(self.logs_bloom)),
                "hash": format!("0x{}", hex::encode(self.hash.as_bytes())),
                "miner": format!("0x{}", hex::encode(self.miner.as_bytes())),
                "mixHash": format!("0x{}", hex::encode(self.mix_hash.as_bytes())),
                "extraData": format!("0x{}", hex::encode(self.extra_data.clone())),
                "stateRoot": format!("0x{}", hex::encode(self.state_root.as_bytes())),
                "parentHash": format!("0x{}", hex::encode(self.parent_hash.as_bytes())),
                "sha3Uncles": format!("0x{}", hex::encode(self.sha3_uncles.as_bytes())),
                "receiptsRoot": format!("0x{}", hex::encode(self.receipts_root.as_bytes())),
                "transactionsRoot": format!("0x{}", hex::encode(self.transactions_root.as_bytes())),
            })
        )
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::eth_test_utils::{
        SAMPLE_RECEIPT_INDEX,
        get_expected_receipt,
        get_sample_log_with_desired_topic,
        get_sample_eth_block_and_receipts,
        get_sample_receipt_with_desired_topic,
        get_sample_eth_block_and_receipts_json,
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

    #[test]
    fn should_encode_eth_block_as_json() {
        let block = get_sample_eth_block_and_receipts().block;
        let uncles: Vec<String> = vec![];
        let expected_result = json!({
            "size": 5774,
            "uncles": uncles,
            "number": 8503804,
            "gasUsed": 7991121,
            "gasLimit": 8003897,
            "timestamp": 1567871882,
            "nonce": "0x9f6d788005a450ed",
            "difficulty": "2273132780410076",
            "totalDifficulty": "11807213944136620030265",
            "miner": "0x5a0b54d5dc17e0aadc383d2db43b0a0d3e029c4c",
            "extraData": "0x5050594520737061726b706f6f6c2d6574682d636e2d687a33",
            "hash": "0xb626a7546311dd56c6f5e9fd07d00c86074077bbd6d5a4c4f8269a2490aa47c0",
            "mixHash": "0xb3a1d476b9632a39df2edd3116692165a7bc363b7f5647c069f54b670cd564ae",
            "stateRoot": "0x061d01dd552a3538b3eadf6234382aeb27cd80cd5cd88b3825fd6990fd762824",
            "parentHash": "0x26e9930dafaf07f59b6c8fe2963819b7d9319ad4ff556cb12eefba0dbd3af3fb",
            "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
            "receiptsRoot": "0x937e08f03388b32d7c776e7a02371b930d71e3ec096d495230b6735e7f9b20ae",
            "transactionsRoot": "0x989081ea9213babd8e82b99b579b3012c3d33434b420c3f97af0e9f6f8b8e047",
            "logsBloom": "0x10040060000810a000180002060000042000328000101012000204800010010000412401000100080012600209a005001200048a0c048008413ca08d8021414000000012002200004880b408400810408000040401c0005000018009804b000480020000122004003200004004080920080020058081444000080a9000a000004080000041100202000000004006040080a80001a12000100000400020340050020080040200200008000082104010040080010481020080000220000124051640075007890200000040c420000820400020800028420018000800020000208080322000000a200008a002000000800101044000000920418600200666900601",
            "transactions": vec![
                "0xee6b2afff6a61686199965dd64d56ec613213b48bb4620e71e0176a881d3b0dc",
                "0xf2df2d51c0b5187e32363ec5dbcfe2e0bb8b8cb70a6708ffc0095d9db53ffda9",
                "0xab8078c9aa8720c5f9206bd2673f25f359d8a01b62212da99ff3b53c1ca3d440",
                "0x0ab2a8d425c3a55855717ce37b0831f644ae8afe496b269b347690ab4f393e3e",
                "0x5af4923b95627fdc57c6573d16e6fa0df716a98063a1027d9733e3eed2cbc24b",
                "0x93c8c513ad5a3eed0150166861c76010254efedbe4951ccb4d02f81cc0f85369",
                "0xe35e3b404ccd568df46ed52ce421998b83063ee1ee1420b36a90288121d5dcc1",
                "0xcdc5a5c943c62a489a04045dbe0e10eda34e3a7162ca6fb0e618b6590ca72ae1",
                "0xe805f3c56e99d3dbbf3bc0fd93f440fd8c9dae1f7876153f96449da523ea21f0",
                "0x4250ff983d0907f560003873c6a916e319a85a111f26127fb2ad459a296e0ce8",
                "0x8cedbb955a7c090ea993591ea541adfe1383f3b2391b74526ef481729b32aa7f",
                "0x8bbcf4950d5924a739114ca0c2bc6f2be118651ccd0dc9028f74f500198ecc06",
                "0x5f023c49e60c14763f5fe72cf6df2666aa4d311e6897ce408301a7246dc17bda",
                "0xbbebd7bbb8797b8790e4f91a0ee49080c4456b8f95c27af8562f70dda40be67a",
                "0x640cb533d56a7e215c6a81aa1cf988c1e7ba479e70a571b974fa811ab2d41796",
                "0xa067162103a794e23234844ff4c8951853488cbafb3e138df2a8ce24968fd394",
                "0xf9ca12a74c3454fcf7e23f5287a057c3605e2aec13fee03a3e03b4774b5faf38",
                "0x20d2a35a89b01589489f142f4881acf8e419308f99c30c791a1bb1f3035b949e",
                "0x40a07797beb2b5247a832e62deff7b631f415a5e6c559eae621d40bc7c33e8bd",
                "0x852cce56dcd2d00c22fab9143d59e5e2a547f0d3390e500f351124b922e7903d",
                "0x164207a34902693be57ccc4b6c2860eb781db2aba1a6e2ed93473a9dd516a542",
                "0x9b8063fe52a38566d5279e8ee9fa3c23c17557b339ea55a7ea1100b44f436434",
                "0x5272da6bc5a763d93e2023a1cd80ad97a112d4a8af0e8e0629c5e7d6e5eddb9d",
                "0x4d2c712ffbc54f8970a4377c03cc7ca8b6d58f8af2181282954b9b16f860cda2",
                "0x49b980475527f989936ddc8afd1e045612cd567238bb567dbd99b48ad15860dc"
            ]
        });
        let result = block.to_json().unwrap();
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

}
