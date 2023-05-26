#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use common_eth::EthSubmissionMaterial;

pub const SAMPLE_SEQUENTIAL_BLOCK_AND_RECEIPT_JSONS_PATH_PREFIX: &str =
    "src/test_utils/sequential_block_and_receipts_jsons/eth_block_and_receipts_num_";

pub const SEQUENTIAL_BLOCKS_FIRST_NUMBER: usize = 8065750;

pub fn get_sequential_eth_blocks_and_receipts() -> Vec<EthSubmissionMaterial> {
    let mut block_and_receipts = Vec::new();
    for i in 0..20 {
        let path = format!(
            "{}{}.json",
            SAMPLE_SEQUENTIAL_BLOCK_AND_RECEIPT_JSONS_PATH_PREFIX,
            SEQUENTIAL_BLOCKS_FIRST_NUMBER + i,
        );
        let block_and_receipt = EthSubmissionMaterial::from_str(&read_to_string(path).unwrap()).unwrap();
        block_and_receipts.push(block_and_receipt)
    }
    block_and_receipts
}
