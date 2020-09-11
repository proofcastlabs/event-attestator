use serde_json::{
    json,
    Value as JsonValue,
};
use ethereum_types::{
    H256 as EthHash,
    Address as EthAddress,
};
use crate::{
    errors::AppError,
    types::{
        Byte,
        Bytes,
        Result,
    },
    chains::eth::{
        eth_block::{
            EthBlock,
            EthBlockJson,
        },
        eth_receipt::{
            EthReceipt,
            EthReceipts,
            EthReceiptJson,
        },
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct EthBlockAndReceipts {
    pub block: EthBlock,
    pub receipts: EthReceipts,
}

impl EthBlockAndReceipts {
    fn new(block: EthBlock, receipts: EthReceipts) -> Self {
        Self { block, receipts }
    }

    pub fn get_receipts(&self) -> Vec<EthReceipt> {
        self.receipts.0.clone()
    }

    pub fn to_json(&self) -> Result<JsonValue> {
        Ok(json!({
            "block": &self.block.to_json()?,
            "receipts": self.receipts.0.iter().map(|receipt| receipt.to_json()).collect::<Result<Vec<JsonValue>>>()?,
        }))
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Self::from_json(&serde_json::from_slice(bytes)?)
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.to_json()?)?)
    }

    pub fn from_json(json: &EthBlockAndReceiptsJson) -> Result<Self> {
        Ok(
            EthBlockAndReceipts {
                block: EthBlock::from_json(&json.block)?,
                receipts: EthReceipts::from_jsons(&json.receipts)?,
            }
        )
    }

    pub fn from_str(json_str: &str) -> Result<Self> {
        Self::from_json(&EthBlockAndReceiptsJson::from_str(json_str)?)
    }

    #[cfg(test)]
    pub fn to_string(&self) -> Result<String> {
        Ok(self.to_json()?.to_string())
    }

    pub fn filter_for_receipts_containing_log_with_address_and_topics(
        &self,
        address: &EthAddress,
        topics: &[EthHash],
    ) -> Result<Self> {
        info!("✔ Number of receipts before filtering: {}", self.receipts.len());
        let filtered = Self::new(
            self.block.clone(),
            self.receipts.filter_for_receipts_containing_log_with_address_and_topics(address, topics),
        );
        info!("✔ Number of receipts after filtering:  {}", filtered.receipts.len());
        Ok(filtered)
    }

    pub fn receipts_are_valid(&self) -> Result<bool> {
        self
            .receipts
            .get_merkle_root()
            .map(|calculated_root| {
                info!("✔    Block's receipts root: {}", self.block.receipts_root.to_string());
                info!("✔ Calculated receipts root: {}", calculated_root.to_string());
                calculated_root == self.block.receipts_root
            })
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct EthBlockAndReceiptsJson {
    pub block: EthBlockJson,
    pub receipts: Vec<EthReceiptJson>
}

impl EthBlockAndReceiptsJson {
    pub fn from_str(json_str: &str) -> Result<Self> {
        match serde_json::from_str(&json_str) {
            Ok(result) => Ok(result),
            Err(e) => Err(AppError::Custom(e.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        chains::eth::eth_constants::REDEEM_EVENT_TOPIC_HEX,
        btc_on_eth::eth::eth_test_utils::{
            get_expected_block,
            get_expected_receipt,
            SAMPLE_RECEIPT_INDEX,
            get_sample_contract_topics,
            get_sample_contract_address,
            get_sample_eth_block_and_receipts,
            get_sample_eth_block_and_receipts_n,
            get_sample_eth_block_and_receipts_string,
        },
    };

    #[test]
    fn should_parse_eth_block_and_receipts_json_string() {
        let json_string = get_sample_eth_block_and_receipts_string(0).unwrap();
        if EthBlockAndReceiptsJson::from_str(&json_string).is_err() {
            panic!("SHould parse eth block and json string correctly!");
        }
    }

    #[test]
    fn should_parse_eth_block_and_receipts_json() {
        let json_string = get_sample_eth_block_and_receipts_string(0).unwrap();
        match EthBlockAndReceipts::from_str(&json_string) {
            Ok(block_and_receipt) => {
                let block = block_and_receipt
                    .block
                    .clone();
                let receipt = block_and_receipt.receipts.0[SAMPLE_RECEIPT_INDEX].clone();
                let expected_block = get_expected_block();
                let expected_receipt = get_expected_receipt();
                assert_eq!(block, expected_block);
                assert_eq!(receipt, expected_receipt);
            }
            _ => panic!("Should parse block & receipts correctly!"),
        }
    }

    #[test]
    fn should_make_to_and_from_string_round_trip() {
        let block_and_receipts = EthBlockAndReceipts::from_str(
            &get_sample_eth_block_and_receipts_string(0).unwrap()
        ).unwrap();
        let string = block_and_receipts.to_string().unwrap();
        let result = EthBlockAndReceipts::from_str(&string).unwrap();
        assert_eq!(result, block_and_receipts);
    }

    #[test]
    fn should_decode_block_and_recipts_json_correctly() {
        let block_and_receipts = get_sample_eth_block_and_receipts();
        let bytes = block_and_receipts.to_bytes().unwrap();
        let result = EthBlockAndReceipts::from_bytes(&bytes).unwrap();
        assert_eq!(result.block, block_and_receipts.block);
        block_and_receipts
            .receipts
            .0
            .iter()
            .enumerate()
            .map(|(i, receipt)| assert_eq!(receipt, &result.receipts.0[i]))
            .for_each(drop);
    }

    #[test]
    fn should_make_to_and_from_bytes_round_trip_correctly() {
        let block_and_receipts = get_sample_eth_block_and_receipts();
        let bytes = block_and_receipts.to_bytes().unwrap();
        let result = EthBlockAndReceipts::from_bytes(&bytes).unwrap();
        assert_eq!(result, block_and_receipts);
    }

    #[test]
    fn should_filter_eth_block_and_receipts() {
        let block_and_receipts = get_sample_eth_block_and_receipts();
        let num_receipts_before = block_and_receipts.receipts.len();
        let address = get_sample_contract_address();
        let topics = get_sample_contract_topics();
        let result = block_and_receipts.filter_for_receipts_containing_log_with_address_and_topics(&address, &topics)
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
            .map(|receipt| assert!(receipt.logs.contain_address(&address)))
            .for_each(drop);
    }

    #[test]
    fn should_filter_eth_block_and_receipts_2() {
        let expected_num_receipts_after = 1;
        let block_and_receipts = get_sample_eth_block_and_receipts_n(6).unwrap();
        let num_receipts_before = block_and_receipts.receipts.len();
        let address = EthAddress::from_slice(&hex::decode("74630cfbc4066726107a4efe73956e219bbb46ab").unwrap());
        let topics = vec![EthHash::from_slice(&hex::decode(REDEEM_EVENT_TOPIC_HEX).unwrap()) ];
        let result = block_and_receipts.filter_for_receipts_containing_log_with_address_and_topics(&address, &topics)
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
            .map(|receipt| assert!(receipt.logs.contain_address(&address)))
            .for_each(drop);
    }

    #[test]
    fn should_return_true_if_receipts_root_is_correct() {
        let block_and_receipts = get_sample_eth_block_and_receipts();
        let result = block_and_receipts.receipts_are_valid().unwrap();
        assert!(result);
    }
}
