#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use bitcoin::{hashes::Hash, Txid};
use ethereum_types::{Address as EthAddress, H256 as EthHash};

use crate::{
    btc_on_eth::{
        eth::{BtcOnEthBtcTxInfo, BtcOnEthBtcTxInfos},
        utils::convert_satoshis_to_wei,
        BtcOnEthEthTxInfo,
        BtcOnEthEthTxInfos,
    },
    chains::{btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS, eth::eth_submission_material::EthSubmissionMaterial},
    types::Result,
};

pub fn get_sample_btc_on_eth_btc_tx_info_1() -> BtcOnEthBtcTxInfo {
    BtcOnEthBtcTxInfo {
        amount_in_satoshis: 123456789,
        recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
        from: EthAddress::from_slice(&hex::decode("7d39fb393c5597dddccf1c428f030913fe7f67ab").unwrap()),
        originating_tx_hash: EthHash::from_slice(
            &hex::decode("01920b62cd2e77204b2fa59932f9d6dd54fd43c99095aee808b700ed2b6ee9cf").unwrap(),
        ),
    }
}

fn get_sample_btc_on_eth_btc_tx_info_2() -> BtcOnEthBtcTxInfo {
    BtcOnEthBtcTxInfo {
        amount_in_satoshis: 987654321,
        recipient: "mudzxCq9aCQ4Una9MmayvJVCF1Tj9fypiM".to_string(),
        from: EthAddress::from_slice(&hex::decode("7d39fb393c5597dddccf1c428f030913fe7f67ab").unwrap()),
        originating_tx_hash: EthHash::from_slice(
            &hex::decode("01920b62cd2e77204b2fa59932f9d6dd54fd43c99095aee808b700ed2b6ee9cf").unwrap(),
        ),
    }
}

pub fn get_sample_btc_on_eth_btc_tx_infos() -> BtcOnEthBtcTxInfos {
    BtcOnEthBtcTxInfos::new(vec![
        get_sample_btc_on_eth_btc_tx_info_1(),
        get_sample_btc_on_eth_btc_tx_info_2(),
    ])
}

pub fn get_sample_btc_on_eth_eth_submission_material_n(num: usize) -> Result<EthSubmissionMaterial> {
    EthSubmissionMaterial::from_str(&read_to_string(format!(
        "src/v1/btc_on_eth/test_utils/eth-sample-submission-material-{}.json",
        num
    ))?)
}

pub fn get_sample_eth_tx_infos() -> BtcOnEthEthTxInfos {
    let originating_tx_address_1 = "335cC6c8e77ECD56402Fa7d4007622A6841a8B6A".to_string();
    let originating_tx_address_2 = "c2f16d5040deDa48Fe9292c183c5D76321e83467".to_string();
    let originating_tx_address_3 = "6635F83421Bf059cd8111f180f0727128685BaE4".to_string();
    let eth_address_1 = EthAddress::from_str(&originating_tx_address_1).unwrap();
    let eth_address_2 = EthAddress::from_str(&originating_tx_address_2).unwrap();
    let eth_address_3 = EthAddress::from_str(&originating_tx_address_3).unwrap();
    let amount_1 = convert_satoshis_to_wei(MINIMUM_REQUIRED_SATOSHIS);
    let amount_2 = convert_satoshis_to_wei(MINIMUM_REQUIRED_SATOSHIS + 1);
    let amount_3 = convert_satoshis_to_wei(MINIMUM_REQUIRED_SATOSHIS - 1);
    let originating_tx_hash_1 = Txid::hash(b"something_1");
    let originating_tx_hash_2 = Txid::hash(b"something_2");
    let originating_tx_hash_3 = Txid::hash(b"something_3");
    let eth_token_address = EthAddress::default();
    let user_data = None;
    let minting_params_1 = BtcOnEthEthTxInfo {
        amount: amount_1,
        destination_address: eth_address_1,
        originating_tx_hash: originating_tx_hash_1,
        originating_tx_address: originating_tx_address_1,
        user_data: user_data.clone(),
        eth_token_address,
    };
    let minting_params_2 = BtcOnEthEthTxInfo {
        amount: amount_2,
        destination_address: eth_address_2,
        originating_tx_hash: originating_tx_hash_2,
        originating_tx_address: originating_tx_address_2,
        user_data: user_data.clone(),
        eth_token_address,
    };
    let minting_params_3 = BtcOnEthEthTxInfo {
        amount: amount_3,
        destination_address: eth_address_3,
        originating_tx_hash: originating_tx_hash_3,
        originating_tx_address: originating_tx_address_3,
        user_data,
        eth_token_address,
    };
    BtcOnEthEthTxInfos::new(vec![minting_params_1, minting_params_2, minting_params_3])
}
