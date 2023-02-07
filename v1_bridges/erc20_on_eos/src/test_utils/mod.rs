#![cfg(test)]
use std::{fs::read_to_string, path::Path, str::FromStr};

use common::{
    chains::{
        eos::eos_submission_material::EosSubmissionMaterial,
        eth::{
            eth_chain_id::EthChainId,
            eth_log::EthLog,
            eth_receipt::EthReceipt,
            eth_submission_material::EthSubmissionMaterial,
        },
    },
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryEntry},
    types::Result,
};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde_json::json;

use crate::eth::{Erc20OnEosEosTxInfo, Erc20OnEosEosTxInfos};

pub fn get_sample_eos_submission_material_n(num: usize) -> EosSubmissionMaterial {
    let s = match num {
        _ => "src/test_utils/mainnet-submission-material-with-perc20-redeem.json",
    };
    EosSubmissionMaterial::from_str(&read_to_string(Path::new(s)).unwrap()).unwrap()
}

// TODO The eth->eos decimal conversion makes this a bad example now. Get a better one!
pub fn get_sample_erc20_on_eos_eos_tx_info() -> Result<Erc20OnEosEosTxInfo> {
    let user_data = vec![];
    Ok(Erc20OnEosEosTxInfo::new(
        U256::from_dec_str("1337").unwrap(),
        EthAddress::from_slice(&hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap()),
        EthAddress::from_slice(&hex::decode("9f57cb2a4f462a5258a49e88b4331068a391de66").unwrap()),
        "aneosaddress".to_string(),
        EthHash::from_slice(&hex::decode("241f386690b715422102edf42f5c9edcddea16b64f17d02bad572f5f341725c0").unwrap()),
        "SampleToken".to_string(),
        "0.000000000 SAM".to_string(),
        user_data,
        EthChainId::Mainnet,
    ))
}

pub fn get_sample_erc20_on_eos_eos_tx_info_2() -> Result<Erc20OnEosEosTxInfo> {
    let user_data = vec![];
    Ok(Erc20OnEosEosTxInfo::new(
        U256::from_dec_str("1337000000000").unwrap(),
        EthAddress::from_slice(&hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap()),
        EthAddress::from_slice(&hex::decode("9e57CB2a4F462a5258a49E88B4331068a391DE66").unwrap()),
        "aneosaddress".to_string(),
        EthHash::from_slice(&hex::decode("241f386690b715422102edf42f5c9edcddea16b64f17d02bad572f5f341725c0").unwrap()),
        "sampletokens".to_string(),
        "0.000001337 SAM".to_string(),
        user_data,
        EthChainId::Mainnet,
    ))
}

pub fn get_sample_erc20_on_eos_eos_tx_info_3() -> Result<Erc20OnEosEosTxInfo> {
    let user_data = vec![];
    Ok(Erc20OnEosEosTxInfo::new(
        U256::from_dec_str("666000000000").unwrap(),
        EthAddress::from_slice(&hex::decode("fedfe2616eb3661cb8fed2782f5f0cc91d59dcac").unwrap()),
        EthAddress::from_slice(&hex::decode("9e57CB2a4F462a5258a49E88B4331068a391DE66").unwrap()),
        "aneosaddress".to_string(),
        EthHash::from_slice(&hex::decode("a35f7386118683a0b37044e19422289564e6daa0c72dca55977637435345baec").unwrap()),
        "sampletokens".to_string(),
        "0.00000666 SAM".to_string(),
        user_data,
        EthChainId::Mainnet,
    ))
}

pub fn get_sample_erc20_on_eos_eos_tx_infos() -> Result<Erc20OnEosEosTxInfos> {
    Ok(Erc20OnEosEosTxInfos::new(vec![get_sample_erc20_on_eos_eos_tx_info()?]))
}

pub fn get_sample_erc20_on_eos_eos_tx_infos_2() -> Erc20OnEosEosTxInfos {
    Erc20OnEosEosTxInfos::new(vec![
        get_sample_erc20_on_eos_eos_tx_info_2().unwrap(),
        get_sample_erc20_on_eos_eos_tx_info_3().unwrap(),
    ])
}

pub fn get_sample_eos_eth_dictionary() -> EosEthTokenDictionary {
    EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::from_str(
        &json!({
            "eth_token_decimals": 18,
            "eos_token_decimals": 8,
            "eth_symbol": "SAM",
            "eos_symbol": "SAM",
            "eth_address": "0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac",
            "eos_address": "sampletoken",
        })
        .to_string(),
    )
    .unwrap()])
}

fn get_sample_eth_submission_material_n(num: usize) -> EthSubmissionMaterial {
    let s = match num {
        1 => "src/test_utils/eth-submission-material-block-8739996-with-erc20-peg-in-event.json",
        _ => "src/test_utils/eth-submission-material-block-11087536-with-erc20-peg-in-event.json",
    };
    EthSubmissionMaterial::from_str(&read_to_string(Path::new(s)).unwrap()).unwrap()
}

fn get_sample_receipt_n(sample_block_num: usize, receipt_index: usize) -> EthReceipt {
    get_sample_eth_submission_material_n(sample_block_num).receipts.0[receipt_index].clone()
}

fn get_sample_log_n(sample_block_num: usize, receipt_index: usize, log_index: usize) -> EthLog {
    get_sample_receipt_n(sample_block_num, receipt_index).logs.0[log_index].clone()
}

pub fn get_sample_log_with_erc20_peg_in_event() -> EthLog {
    get_sample_log_n(1, 17, 1)
}

pub fn get_sample_log_with_erc20_peg_in_event_2() -> EthLog {
    get_sample_log_n(2, 16, 1)
}

pub fn get_sample_receipt_with_erc20_peg_in_event() -> EthReceipt {
    get_sample_receipt_n(1, 17)
}

pub fn get_sample_submission_material_with_erc20_peg_in_event() -> EthSubmissionMaterial {
    get_sample_eth_submission_material_n(1)
}
