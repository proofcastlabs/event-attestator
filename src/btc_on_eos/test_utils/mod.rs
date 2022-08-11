#![cfg(test)]
use bitcoin::hashes::{sha256d, Hash};

use crate::{
    btc_on_eos::{
        btc::{BtcOnEosEosTxInfo, BtcOnEosEosTxInfos},
        eos::{BtcOnEosBtcTxInfo, BtcOnEosBtcTxInfos},
        utils::convert_u64_to_x_decimal_eos_asset,
    },
    chains::{
        btc::btc_constants::{BTC_NUM_DECIMALS, MINIMUM_REQUIRED_SATOSHIS},
        eos::eos_test_utils::get_sample_eos_submission_material_n,
    },
};

pub fn get_sample_btc_tx_info() -> BtcOnEosBtcTxInfo {
    let action_proof = get_sample_eos_submission_material_n(1).action_proofs[0].clone();
    BtcOnEosBtcTxInfo::from_action_proof(&action_proof).unwrap()
}

pub fn get_sample_btc_tx_infos() -> BtcOnEosBtcTxInfos {
    BtcOnEosBtcTxInfos::new(vec![get_sample_btc_tx_info(), get_sample_btc_tx_info()])
}

pub fn get_sample_btc_on_eos_eos_tx_infos() -> BtcOnEosEosTxInfos {
    let symbol = "PBTC".to_string();
    let originating_tx_address_1 = "eosaccount1x".to_string();
    let originating_tx_address_2 = "eosaccount2x".to_string();
    let originating_tx_address_3 = "eosaccount3x".to_string();
    let eos_address_1 = originating_tx_address_1.clone();
    let eos_address_2 = originating_tx_address_2.clone();
    let eos_address_3 = originating_tx_address_3.clone();
    let amount_1 = convert_u64_to_x_decimal_eos_asset(MINIMUM_REQUIRED_SATOSHIS, BTC_NUM_DECIMALS, &symbol);
    let amount_2 = convert_u64_to_x_decimal_eos_asset(MINIMUM_REQUIRED_SATOSHIS + 1, BTC_NUM_DECIMALS, &symbol);
    let amount_3 = convert_u64_to_x_decimal_eos_asset(MINIMUM_REQUIRED_SATOSHIS - 1, BTC_NUM_DECIMALS, &symbol);
    let originating_tx_hash_1 = sha256d::Hash::hash(b"something_1").to_string();
    let originating_tx_hash_2 = sha256d::Hash::hash(b"something_2").to_string();
    let originating_tx_hash_3 = sha256d::Hash::hash(b"something_3").to_string();
    let user_data = None;
    let eos_token_address = "anaddress".to_string();
    let minting_params_1 = BtcOnEosEosTxInfo {
        amount: amount_1,
        destination_address: eos_address_1,
        originating_tx_hash: originating_tx_hash_1,
        originating_tx_address: originating_tx_address_1,
        user_data: user_data.clone(),
        eos_token_address: eos_token_address.clone(),
    };
    let minting_params_2 = BtcOnEosEosTxInfo {
        amount: amount_2,
        destination_address: eos_address_2,
        originating_tx_hash: originating_tx_hash_2,
        originating_tx_address: originating_tx_address_2,
        user_data: user_data.clone(),
        eos_token_address: eos_token_address.clone(),
    };
    let minting_params_3 = BtcOnEosEosTxInfo {
        amount: amount_3,
        destination_address: eos_address_3,
        originating_tx_hash: originating_tx_hash_3,
        originating_tx_address: originating_tx_address_3,
        user_data,
        eos_token_address: eos_token_address.clone(),
    };
    BtcOnEosEosTxInfos::new(vec![minting_params_1, minting_params_2, minting_params_3])
}
