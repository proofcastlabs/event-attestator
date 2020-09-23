use std::{
    path::Path,
    fs::read_to_string,
};
use crate::{
    types::Result,
    chains::eth::{
        eth_submission_material::EthSubmissionMaterial,
    },
};

pub const PERC20_SUBMISSION_MATERIAL_1: &str = "src/erc20_on_eos/eth/eth_test_utils/eth-block-10828316-with-dai-transfer-event.json";

pub fn get_sample_eth_submission_material_string(num: usize) -> Result<String> {
    let path = match num {
        0 => Ok(PERC20_SUBMISSION_MATERIAL_1),
        _ => Err(format!("Cannot find sample block num: {}", num))
    }?;
    match Path::new(&path).exists() {
        true => Ok(read_to_string(path)?),
        false => Err("✘ Cannot find sample-eth-block-and-receipts-json file!".into())
    }
}

pub fn get_sample_eth_submission_material_n(num: usize) -> Result<EthSubmissionMaterial> {
    get_sample_eth_submission_material_string(num).and_then(|string| EthSubmissionMaterial::from_str(&string))
}
