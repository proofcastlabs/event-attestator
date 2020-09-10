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
    types::Result,
    btc_on_eth::utils::{
        convert_hex_to_h256,
        convert_hex_to_address,
        convert_json_value_to_string,
    },
    chains::eth::{
        eth_log::{
            EthLogs,
            EthLogJson,
        },
    },
};

#[derive(Debug)]
pub struct EthReceipts(pub Vec<EthReceipt>);

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eth::eth::eth_test_utils::{
        SAMPLE_RECEIPT_INDEX,
        get_expected_receipt,
        get_sample_eth_block_and_receipts,
        get_sample_receipt_with_desired_topic,
        get_sample_eth_block_and_receipts_json,
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
}
