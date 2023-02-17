use common::{types::Result, utils::strip_hex_prefix};
use ethereum_types::{Bloom, U256};
use serde::{Deserialize, Serialize};

use crate::{
    eth_block::EthBlock,
    eth_receipt::EthReceipts,
    eth_submission_material::EthSubmissionMaterial,
    eth_utils::{
        convert_hex_strings_to_h256s,
        convert_hex_to_bytes,
        convert_hex_to_eth_address,
        convert_hex_to_h256,
        decode_prefixed_hex,
    },
};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EthBlockJsonFromRpc {
    size: String,
    hash: String,
    miner: String,
    nonce: String,
    number: String,
    gas_used: String,
    mix_hash: String,
    gas_limit: String,
    timestamp: String,
    logs_bloom: String,
    extra_data: String,
    difficulty: String,
    state_root: String,
    parent_hash: String,
    sha3_uncles: String,
    uncles: Vec<String>,
    receipts_root: String,
    total_difficulty: String,
    transactions_root: String,
    transactions: Vec<String>,
    base_fee_per_gas: Option<String>,
}

impl EthSubmissionMaterial {
    pub fn from_rpc(rpc_block: EthBlockJsonFromRpc) -> Result<Self> {
        EthBlock::from_json_rpc(&rpc_block).map(|block| {
            Self {
                hash: Some(block.hash),
                eos_ref_block_num: None,
                eos_ref_block_prefix: None,
                algo_first_valid_round: None,
                block_number: Some(block.number),
                parent_hash: Some(block.parent_hash),
                receipts_root: Some(block.receipts_root),
                block: Some(block),
                // FIXME
                receipts: EthReceipts(vec![]), // HOW TO GET THESE??
            }
        })
    }
}

impl EthBlock {
    pub fn from_json_rpc(json: &EthBlockJsonFromRpc) -> Result<Self> {
        let radix = 16;
        Ok(EthBlock {
            hash: convert_hex_to_h256(&json.hash)?,
            nonce: decode_prefixed_hex(&json.nonce)?,
            miner: convert_hex_to_eth_address(&json.miner)?,
            mix_hash: convert_hex_to_h256(&json.mix_hash)?,
            state_root: convert_hex_to_h256(&json.state_root)?,
            extra_data: convert_hex_to_bytes(&json.extra_data)?,
            parent_hash: convert_hex_to_h256(&json.parent_hash)?,
            sha3_uncles: convert_hex_to_h256(&json.sha3_uncles)?,
            receipts_root: convert_hex_to_h256(&json.receipts_root)?,
            transactions_root: convert_hex_to_h256(&json.transactions_root)?,
            size: U256::from_str_radix(&strip_hex_prefix(&json.size), radix)?,
            number: U256::from_str_radix(&strip_hex_prefix(&json.number), radix)?,
            gas_used: U256::from_str_radix(&strip_hex_prefix(&json.gas_used), radix)?,
            gas_limit: U256::from_str_radix(&strip_hex_prefix(&json.gas_limit), radix)?,
            logs_bloom: Bloom::from_slice(&convert_hex_to_bytes(&json.logs_bloom)?[..]),
            timestamp: U256::from_str_radix(&strip_hex_prefix(&json.timestamp), radix)?,
            difficulty: U256::from_str_radix(&strip_hex_prefix(&json.difficulty), radix)?,
            uncles: convert_hex_strings_to_h256s(json.uncles.iter().map(AsRef::as_ref).collect())?,
            total_difficulty: U256::from_str_radix(&strip_hex_prefix(&json.total_difficulty), radix)?,
            transactions: json
                .transactions
                .iter()
                .map(|s| convert_hex_to_h256(s))
                .collect::<Result<Vec<_>>>()?,
            base_fee_per_gas: match json.base_fee_per_gas.as_ref() {
                None => None,
                Some(hex) => Some(U256::from_str_radix(&strip_hex_prefix(hex), radix)?),
            },
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EthReceiptJsonFromRpc {
    v: String,
    r: String,
    s: String,
    to: String,
    gas: String,
    hash: String,
    from: String,
    value: String,
    input: String,
    nonce: String,
    gas_price: String,
    block_hash: String,
    block_number: String,
    status: Option<String>,
    chain_id: Option<String>,
    transaction_index: String,
    #[serde(rename = "type")]
    receipt_type: String,
}

/* TODO finish this?
impl EthReceipt {
    pub fn from_json(json: &EthReceiptJsonFromRpc) -> Result<Self> {
        let logs = EthLogs::from_receipt_json(json)?;
        Ok(Self {
            status: match json.status {
                Some(x) if x == "0x1" || x == "0x01"  => true,
                _ => false,
            },
            /*
            logs_bloom: logs.get_bloom(),
            gas_used: U256::from(json.gas_used),
            from: convert_hex_to_eth_address(&json.from)?,
            block_number: U256::from(json.block_number),
            block_hash: convert_hex_to_h256(&json.block_hash)?,
            transaction_index: U256::from(json.transaction_index),
            cumulative_gas_used: U256::from(json.cumulative_gas_used),
            transaction_hash: convert_hex_to_h256(&json.transaction_hash)?,
            to: match json.to {
                serde_json::Value::Null => H160::zero(),
                _ => convert_hex_to_eth_address(&convert_json_value_to_string(&json.to)?)?,
            },
            contract_address: match json.contract_address {
                serde_json::Value::Null => EthAddress::zero(),
                _ => convert_hex_to_eth_address(&convert_json_value_to_string(&json.contract_address)?)?,
            },
            receipt_type: match json.receipt_type {
                Some(ref hex) => Some(EthReceiptType::from_byte(&hex::decode(strip_hex_prefix(hex))?[0])),
                None => None,
            },
            logs,
            */
        })
    }
}
*/

#[cfg(test)]
mod tests {
    use common::EthChainId;

    use super::*;
    use crate::test_utils::get_sample_block_from_rpc;

    #[test]
    fn should_get_block_from_block_from_rpc_json() {
        let s = get_sample_block_from_rpc();
        let json = EthBlockJsonFromRpc::from_str(&s).unwrap();
        let result = EthBlock::from_json_rpc(&json).unwrap();
        let chain_id = EthChainId::Mainnet;
        let is_valid = result.is_valid(&chain_id).unwrap();
        assert!(is_valid);
    }
}
