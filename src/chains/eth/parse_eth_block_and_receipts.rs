use crate::{
    types::Result,
    errors::AppError,
    chains::eth::{
        eth_block::EthBlock,
        eth_receipt::EthReceipts,
        eth_types::{
            EthBlockAndReceipts,
            EthBlockAndReceiptsJson,
        },
    },
};

pub fn parse_eth_block_and_receipts_json_string(json_string: &str) -> Result<EthBlockAndReceiptsJson> {
    match serde_json::from_str(&json_string) {
        Ok(result) => Ok(result),
        Err(e) => Err(AppError::Custom(e.to_string()))
    }
}

pub fn parse_eth_block_and_receipts_json(json: EthBlockAndReceiptsJson) -> Result<EthBlockAndReceipts> {
    Ok(
        EthBlockAndReceipts {
            block: EthBlock::from_json(&json.block)?,
            receipts: EthReceipts::from_jsons(&json.receipts)?.0,
        }
    )
}

pub fn parse_eth_block_and_receipts(eth_block_and_receipts: &str) -> Result<EthBlockAndReceipts> {
    parse_eth_block_and_receipts_json_string(eth_block_and_receipts)
        .and_then(parse_eth_block_and_receipts_json)
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
        if parse_eth_block_and_receipts_json_string(&json_string).is_err() {
            panic!("SHould parse eth block and json string correctly!");
        }
    }

    #[test]
    fn should_parse_eth_block_and_receipts_json() {
        let json_string = get_sample_eth_block_and_receipts_string(0).unwrap();
        match parse_eth_block_and_receipts(&json_string) {
            Ok(block_and_receipt) => {
                let block = block_and_receipt
                    .block
                    .clone();
                let receipt = block_and_receipt
                    .receipts[SAMPLE_RECEIPT_INDEX].clone();
                let expected_block = get_expected_block();
                let expected_receipt = get_expected_receipt();
                assert_eq!(block, expected_block);
                assert_eq!(receipt, expected_receipt);
            }
            _ => panic!("Should parse block & receipt correctly!"),
        }
    }

    #[test]
    fn should_parse_eth_block_and_receipts() {
        let json_string = get_sample_eth_block_and_receipts_string(0).unwrap();
        match parse_eth_block_and_receipts(&json_string) {
            Ok(block_and_receipt) => {
                let block = block_and_receipt
                    .block
                    .clone();
                let receipt = block_and_receipt
                    .receipts[SAMPLE_RECEIPT_INDEX].clone();
                let expected_block = get_expected_block();
                let expected_receipt = get_expected_receipt();
                assert_eq!(block, expected_block);
                assert_eq!(receipt, expected_receipt);
            }
            _ => panic!("Should parse block & receipt correctly!"),
        }
    }
}
