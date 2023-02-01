#![cfg(test)]

use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde_json::json;

use crate::{
    chains::eth::eth_chain_id::EthChainId,
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryEntry},
    erc20_on_eos::{Erc20OnEosEosTxInfo, Erc20OnEosEosTxInfos},
    types::Result,
};

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
