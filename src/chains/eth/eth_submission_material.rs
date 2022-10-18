#![allow(clippy::manual_map)]
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::Deserialize;
use serde_json::{json, Value as JsonValue};

use crate::{
    chains::eth::{
        eth_block::{EthBlock, EthBlockJson},
        eth_receipt::{EthReceipt, EthReceiptJson, EthReceipts},
        eth_state::EthState,
    },
    traits::DatabaseInterface,
    types::{Byte, Bytes, NoneError, Result},
};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EthSubmissionMaterial {
    pub block: Option<EthBlock>,
    pub receipts: EthReceipts,
    pub eos_ref_block_num: Option<u16>,
    pub eos_ref_block_prefix: Option<u32>,
    pub hash: Option<EthHash>,
    pub block_number: Option<U256>,
    pub parent_hash: Option<EthHash>,
    pub receipts_root: Option<EthHash>,
    pub algo_first_valid_round: Option<u64>,
}

impl EthSubmissionMaterial {
    fn init(
        block: EthBlock,
        receipts: EthReceipts,
        eos_ref_block_num: Option<u16>,
        eos_ref_block_prefix: Option<u32>,
        algo_first_valid_round: Option<u64>,
    ) -> Self {
        Self {
            receipts,
            algo_first_valid_round,
            eos_ref_block_num,
            eos_ref_block_prefix,
            hash: Some(block.hash),
            block_number: Some(block.number),
            parent_hash: Some(block.parent_hash),
            receipts_root: Some(block.receipts_root),
            block: Some(block),
        }
    }

    pub fn new(
        block: EthBlock,
        receipts: EthReceipts,
        eos_ref_block_num: Option<u16>,
        eos_ref_block_prefix: Option<u32>,
    ) -> Self {
        Self::init(block, receipts, eos_ref_block_num, eos_ref_block_prefix, None)
    }

    pub fn get_algo_first_valid_round(&self) -> Result<u64> {
        match self.algo_first_valid_round {
            Some(round) => Ok(round),
            None => Err("No `algo_first_valid_round` in `EthSubmissionMaterial`!".into()),
        }
    }

    pub fn get_block(&self) -> Result<EthBlock> {
        self.block
            .clone()
            .ok_or_else(|| NoneError("✘ No block in ETH submisson material!"))
    }

    pub fn get_block_hash(&self) -> Result<EthHash> {
        self.hash
            .ok_or_else(|| NoneError("✘ No `hash` in ETH submission material!"))
    }

    pub fn get_parent_hash(&self) -> Result<EthHash> {
        self.parent_hash
            .ok_or_else(|| NoneError("✘ No` parent_hash` in ETH submission material!"))
    }

    pub fn get_block_number(&self) -> Result<U256> {
        self.block_number
            .ok_or_else(|| NoneError("✘ No `block_number` in ETH submission material!"))
    }

    pub fn get_receipts_root(&self) -> Result<EthHash> {
        self.receipts_root
            .ok_or_else(|| NoneError("✘ No `receipts_root` in ETH submission material!"))
    }

    pub fn get_eos_ref_block_num(&self) -> Result<u16> {
        self.eos_ref_block_num
            .ok_or_else(|| NoneError("No `eos_ref_block_num` in submission material!"))
    }

    pub fn get_eos_ref_block_prefix(&self) -> Result<u32> {
        self.eos_ref_block_prefix
            .ok_or_else(|| NoneError("No `eos_ref_block_prefix` in submission material!"))
    }

    pub fn get_receipts(&self) -> Vec<EthReceipt> {
        self.receipts.0.clone()
    }

    pub fn get_num_receipts(&self) -> usize {
        self.receipts.len()
    }

    pub fn to_json(&self) -> Result<JsonValue> {
        let block_json = match &self.block {
            Some(block) => Some(block.to_json()?),
            None => None,
        };
        Ok(json!({
            "hash": self.hash,
            "block": block_json,
            "parent_hash": self.parent_hash,
            "block_number": self.block_number,
            "receipts_root": self.receipts_root,
            "eos_ref_block_num": self.eos_ref_block_num,
            "eos_ref_block_prefix": self.eos_ref_block_prefix,
            "receipts": self.receipts.0.iter().map(|receipt| receipt.to_json()).collect::<Result<Vec<JsonValue>>>()?,
        }))
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Self::from_json(&serde_json::from_slice(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.to_json()?)?)
    }

    pub fn from_json(json: &EthSubmissionMaterialJson) -> Result<Self> {
        // NOTE: Legacy cores originally stored the full block. To reduce the size of the encrypted DB,
        // cores v1.19.0 and later remove the ETH block when saving to the db. Hence why here we
        // first check if there *is* a block in the json retrieved from the DB and then create the correct
        // (new) struct that way. Otherwise, we check the json correctly adheres to the new format
        // and if so create the struct from that instead.
        let block = match json.block {
            Some(ref block_json) => Some(EthBlock::from_json(block_json)?),
            None => None,
        };
        let receipts = EthReceipts::from_jsons(&json.receipts.clone())?;
        match block {
            Some(block) => Ok(EthSubmissionMaterial {
                receipts,
                hash: Some(block.hash),
                block_number: Some(block.number),
                parent_hash: Some(block.parent_hash),
                receipts_root: Some(block.receipts_root),
                eos_ref_block_num: json.eos_ref_block_num,
                eos_ref_block_prefix: json.eos_ref_block_prefix,
                block: Some(block),
                algo_first_valid_round: json.algo_first_valid_round,
            }),
            None => {
                if json.hash.is_none() {
                    return Err("Error parsing `EthSubmissionInfo` from json: missing `hash`!".into());
                } else if json.parent_hash.is_none() {
                    return Err("Error parsing `EthSubmissionInfo` from json: missing `parent_hash`!".into());
                } else if json.block_number.is_none() {
                    return Err("Error parsing `EthSubmissionInfo` from json: missing `block_number`!".into());
                } else if json.receipts_root.is_none() {
                    return Err("Error parsing `EthSubmissionInfo` from json: missing `receipts_root`!".into());
                };
                Ok(EthSubmissionMaterial {
                    receipts,
                    block: None,
                    hash: json.hash,
                    parent_hash: json.parent_hash,
                    block_number: json.block_number,
                    receipts_root: json.receipts_root,
                    eos_ref_block_num: json.eos_ref_block_num,
                    eos_ref_block_prefix: json.eos_ref_block_prefix,
                    algo_first_valid_round: json.algo_first_valid_round,
                })
            },
        }
    }

    pub fn from_str(json_str: &str) -> Result<Self> {
        info!("✔ Parsing ETH submission material...");
        let submission_material = Self::from_json(&EthSubmissionMaterialJson::from_str(json_str)?)?;
        info!(
            "✔ ETH submission material parsed! Block number: {}",
            submission_material.get_block_number()?
        );
        Ok(submission_material)
    }

    #[cfg(test)]
    pub fn to_string(&self) -> Result<String> {
        Ok(self.to_json()?.to_string())
    }

    pub fn get_receipts_containing_log_from_address_and_with_topics(
        &self,
        address: &EthAddress,
        topics: &[EthHash],
    ) -> Result<Self> {
        info!("✔ Number of receipts before filtering: {}", self.receipts.len());
        let receipts_after = self
            .receipts
            .get_receipts_containing_log_from_address_and_with_topics(address, topics);
        let mut mutable_self = self.clone();
        mutable_self.receipts = receipts_after;
        info!("✔ Number of receipts after filtering: {}", mutable_self.receipts.len());
        Ok(mutable_self)
    }

    pub fn get_receipts_containing_log_from_addresses_and_with_topics(
        &self,
        addresses: &[EthAddress],
        topics: &[EthHash],
    ) -> Result<Self> {
        info!("✔ Number of receipts before filtering: {}", self.receipts.len());
        let receipts_after = self
            .receipts
            .get_receipts_containing_log_from_addresses_and_with_topics(addresses, topics);
        info!("✔ Number of receipts after filtering:  {}", receipts_after.len());
        let mut mutable_self = self.clone();
        mutable_self.receipts = receipts_after;
        Ok(mutable_self)
    }

    pub fn receipts_are_valid(&self) -> Result<bool> {
        self.receipts.get_merkle_root().and_then(|calculated_root| {
            let receipts_root = self.get_receipts_root()?;
            info!("✔    Block's receipts root: {}", receipts_root.to_string());
            info!("✔ Calculated receipts root: {}", calculated_root.to_string());
            Ok(calculated_root == receipts_root)
        })
    }

    pub fn remove_receipts(&self) -> Self {
        let mut mutable_self = self.clone();
        mutable_self.receipts = vec![].into();
        mutable_self
    }

    pub fn remove_block(&self) -> Self {
        let mut mutable_self = self.clone();
        mutable_self.block = None;
        mutable_self
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct EthSubmissionMaterialJson {
    pub block: Option<EthBlockJson>,
    pub receipts: Vec<EthReceiptJson>,
    pub eos_ref_block_num: Option<u16>,
    pub eos_ref_block_prefix: Option<u32>,
    pub hash: Option<EthHash>,
    pub block_number: Option<U256>,
    pub parent_hash: Option<EthHash>,
    pub receipts_root: Option<EthHash>,
    pub algo_first_valid_round: Option<u64>,
}

impl EthSubmissionMaterialJson {
    pub fn from_str(json_str: &str) -> Result<Self> {
        match serde_json::from_str(json_str) {
            Ok(result) => Ok(result),
            Err(e) => Err(e.into()),
        }
    }
}

pub fn parse_eth_submission_material_and_put_in_state<'a, D: DatabaseInterface>(
    block_json: &str,
    state: EthState<'a, D>,
) -> Result<EthState<'a, D>> {
    info!("✔ Parsing ETH block & receipts...");
    EthSubmissionMaterial::from_str(block_json).and_then(|result| state.add_eth_submission_material(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chains::eth::{
        eth_contracts::erc777_token::ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA_HEX,
        eth_test_utils::{
            get_expected_block,
            get_expected_receipt,
            get_sample_contract_address,
            get_sample_contract_topics,
            get_sample_eip1559_mainnet_submission_material,
            get_sample_eip1559_ropsten_submission_material,
            get_sample_eip2718_ropsten_submission_material,
            get_sample_eth_submission_material,
            get_sample_eth_submission_material_n,
            get_sample_eth_submission_material_string,
            SAMPLE_RECEIPT_INDEX,
        },
    };

    #[test]
    fn should_parse_eth_submission_material_json_string() {
        let json_string = get_sample_eth_submission_material_string(0).unwrap();
        if EthSubmissionMaterial::from_str(&json_string).is_err() {
            panic!("SHould parse eth block and json string correctly!");
        }
    }

    #[test]
    fn should_parse_eth_submission_material_json() {
        let json_string = get_sample_eth_submission_material_string(0).unwrap();
        let submission_material = EthSubmissionMaterial::from_str(&json_string).unwrap();
        let block = submission_material.get_block().unwrap();
        let receipt = submission_material.receipts.0[SAMPLE_RECEIPT_INDEX].clone();
        let expected_block = get_expected_block();
        let expected_receipt = get_expected_receipt();
        assert_eq!(block, expected_block);
        assert_eq!(receipt, expected_receipt);
    }

    #[test]
    fn should_make_to_and_from_string_round_trip() {
        let block_and_receipts =
            EthSubmissionMaterial::from_str(&get_sample_eth_submission_material_string(0).unwrap()).unwrap();
        let string = block_and_receipts.to_string().unwrap();
        let result = EthSubmissionMaterial::from_str(&string).unwrap();
        assert_eq!(result, block_and_receipts);
    }

    #[test]
    fn should_decode_block_and_recipts_json_correctly() {
        let block_and_receipts = get_sample_eth_submission_material();
        let bytes = block_and_receipts.to_bytes().unwrap();
        let result = EthSubmissionMaterial::from_bytes(&bytes).unwrap();
        assert_eq!(result.block, block_and_receipts.block);
        block_and_receipts
            .receipts
            .0
            .iter()
            .enumerate()
            .for_each(|(i, receipt)| assert_eq!(receipt, &result.receipts.0[i]));
    }

    #[test]
    fn should_make_to_and_from_bytes_round_trip_correctly() {
        let block_and_receipts = get_sample_eth_submission_material();
        let bytes = block_and_receipts.to_bytes().unwrap();
        let result = EthSubmissionMaterial::from_bytes(&bytes).unwrap();
        assert_eq!(result, block_and_receipts);
    }

    #[test]
    fn should_filter_eth_submission_material() {
        let block_and_receipts = get_sample_eth_submission_material();
        let num_receipts_before = block_and_receipts.receipts.len();
        let address = get_sample_contract_address();
        let topics = get_sample_contract_topics();
        let result = block_and_receipts
            .get_receipts_containing_log_from_address_and_with_topics(&address, &topics)
            .unwrap();
        let num_receipts_after = result.receipts.len();
        assert!(num_receipts_before > num_receipts_after);
        result
            .receipts
            .0
            .iter()
            .map(|receipt| {
                assert!(receipt.logs.contain_topic(&topics[0]));
                receipt
            })
            .for_each(|receipt| assert!(receipt.logs.contain_address(&address)));
    }

    #[test]
    fn should_filter_eth_submission_material_2() {
        let expected_num_receipts_after = 1;
        let block_and_receipts = get_sample_eth_submission_material_n(6).unwrap();
        let num_receipts_before = block_and_receipts.receipts.len();
        let address = EthAddress::from_slice(&hex::decode("74630cfbc4066726107a4efe73956e219bbb46ab").unwrap());
        let topics = vec![EthHash::from_slice(
            &hex::decode(ERC_777_REDEEM_EVENT_TOPIC_WITHOUT_USER_DATA_HEX).unwrap(),
        )];
        let result = block_and_receipts
            .get_receipts_containing_log_from_address_and_with_topics(&address, &topics)
            .unwrap();
        let num_receipts_after = result.receipts.len();
        assert!(num_receipts_before > num_receipts_after);
        assert_eq!(num_receipts_after, expected_num_receipts_after);
        result
            .receipts
            .0
            .iter()
            .map(|receipt| {
                assert!(receipt.logs.contain_topic(&topics[0]));
                receipt
            })
            .for_each(|receipt| assert!(receipt.logs.contain_address(&address)));
    }

    #[test]
    fn should_return_true_if_receipts_root_is_correct() {
        let block_and_receipts = get_sample_eth_submission_material();
        let result = block_and_receipts.receipts_are_valid().unwrap();
        assert!(result);
    }

    #[test]
    fn should_remove_receipts_from_block_and_receipts() {
        let block_and_receipts = get_sample_eth_submission_material();
        let num_receipts_before = block_and_receipts.receipts.len();
        assert!(num_receipts_before > 0);
        let result = block_and_receipts.remove_receipts();
        let num_receipts_after = result.receipts.len();
        assert_eq!(num_receipts_after, 0);
    }

    #[test]
    fn mainnet_eip1559_blocks_receipts_should_be_valid() {
        let submission_material = get_sample_eip1559_mainnet_submission_material();
        let result = submission_material.receipts_are_valid().unwrap();
        assert!(result);
    }

    #[test]
    fn ropsten_eip1559_blocks_receipts_should_be_valid() {
        let submission_material = get_sample_eip1559_ropsten_submission_material();
        let result = submission_material.receipts_are_valid().unwrap();
        assert!(result);
    }

    #[test]
    fn ropsten_block_with_one_eip2718_tx_should_be_valid() {
        let submission_material = get_sample_eip2718_ropsten_submission_material();
        let result = submission_material.receipts_are_valid().unwrap();
        assert!(result);
    }

    #[test]
    fn receipts_roots_of_eth_submission_material_should_be_valid() {
        for i in 0..15 {
            let submission_material = get_sample_eth_submission_material_n(i).unwrap();
            let receipts = submission_material.receipts.clone();
            let expected_root = submission_material.block.unwrap().receipts_root;
            let calculated_root = receipts.get_merkle_root().unwrap();
            assert_eq!(expected_root, calculated_root);
        }
    }
}
