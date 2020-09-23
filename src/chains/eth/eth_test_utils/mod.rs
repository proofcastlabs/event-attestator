use std::{
    path::Path,
    fs::read_to_string,
};

use crate::{
    types::Result,
    chains::eth::{
        eth_log::EthLog,
        eth_receipt::EthReceipt,
        eth_submission_material::EthSubmissionMaterial,
    },
};

pub const SAMPLE_ETH_SUBMISSION_MATERIAL_0: &str = "src/chains/eth/eth_test_utils/eth-submission-material-block-8739996-with-erc20-peg-in-event.json";

pub fn get_sample_eth_submission_material_string_n(n: usize) -> Result<String> {
    let path = match n {
        0 => Ok(SAMPLE_ETH_SUBMISSION_MATERIAL_0),
        _ => Err(format!("Cannot find sample eth submission material num: {}", n))
    }?;
    match Path::new(&path).exists() {
        true => Ok(read_to_string(path)?),
        false => Err(format!("✘ Cannot find file @ '{}'!", path).into())
    }
}

pub fn get_sample_eth_submission_material_n(n: usize) -> Result<EthSubmissionMaterial> {
    get_sample_eth_submission_material_string_n(n).and_then(|string| EthSubmissionMaterial::from_str(&string))
}

pub fn get_sample_receipt_n(n: usize, receipt_index: usize) -> Result<EthReceipt> {
    get_sample_eth_submission_material_n(n).map(|block| block.receipts.0[receipt_index].clone())
}

pub fn get_sample_log_n(n: usize, receipt_index: usize, log_index: usize) -> Result<EthLog> {
    get_sample_receipt_n(n, receipt_index).map(|receipt| receipt.logs.0[log_index].clone())
}

pub fn get_sample_receipt_with_erc20_peg_in_event() -> Result<EthReceipt> {
    get_sample_receipt_n(0, 17)
}

pub fn get_sample_log_with_erc20_peg_in_event() -> Result<EthLog> {
    get_sample_log_n(0, 17, 1)
}
