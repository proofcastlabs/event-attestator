use serde_json::{
    json,
    Value as JsonValue,
};
use crate::{
    errors::AppError,
    types::{
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
    pub fn get_receipts(&self) -> Vec<EthReceipt> {
        self.receipts.0.clone()
    }

    pub fn to_json(&self) -> Result<JsonValue> {
        Ok(json!({
            "block": &self.block.to_json()?,
            "receipts": self.receipts.0.iter().map(|receipt| receipt.to_json()).collect::<Result<Vec<JsonValue>>>()?,
        }))
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
    use crate::btc_on_eth::{
        eth::eth_test_utils::{
            get_expected_block,
            get_expected_receipt,
            SAMPLE_RECEIPT_INDEX,
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
            _ => panic!("Should parse block & receipt correctly!"),
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
}
