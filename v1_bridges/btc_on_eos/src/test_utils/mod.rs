#![cfg(test)]
use std::{fs::read_to_string, path::Path, str::FromStr};

use common_btc::{
    create_unsigned_utxo_from_tx,
    BtcBlockAndId,
    BtcSubmissionMaterialJson,
    BtcUtxoAndValue,
    DepositAddressInfoJson,
    BTC_NUM_DECIMALS,
    MINIMUM_REQUIRED_SATOSHIS,
};
use common_eos::{EosSubmissionMaterial, EosSubmissionMaterialJson};

use crate::{
    bitcoin_crate_alias::hashes::{sha256d, Hash},
    btc::{BtcOnEosEosTxInfo, BtcOnEosEosTxInfos},
    eos::{BtcOnEosBtcTxInfo, BtcOnEosBtcTxInfos},
    utils::convert_u64_to_x_decimal_eos_asset,
};

pub fn get_sample_eos_submission_material_string_n(n: usize) -> String {
    let p = match n {
        1 => "src/test_utils/eos-block-81784220.json",
        2 => "src/test_utils/eos-block-80440580.json",
        3 => "src/test_utils/eos-block-84187467.json",
        4 => "src/test_utils/eos-block-81772484.json",
        _ => "",
    };
    read_to_string(Path::new(p)).unwrap()
}

pub fn get_sample_eos_submission_material_json_n(n: usize) -> EosSubmissionMaterialJson {
    EosSubmissionMaterialJson::from_str(&get_sample_eos_submission_material_string_n(n)).unwrap()
}

pub fn get_sample_eos_submission_material_n(n: usize) -> EosSubmissionMaterial {
    EosSubmissionMaterial::from_str(&get_sample_eos_submission_material_string_n(n)).unwrap()
}

pub fn get_sample_btc_tx_info() -> BtcOnEosBtcTxInfo {
    let submat = get_sample_eos_submission_material_n(1);
    let action_proof = submat.action_proofs[0].clone();
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
        eos_token_address,
    };
    BtcOnEosEosTxInfos::new(vec![minting_params_1, minting_params_2, minting_params_3])
}

pub fn get_sample_p2sh_utxo_and_value_2() -> BtcUtxoAndValue {
    let s = "src/test_utils/btc-1670411-block-and-txs.json";
    let block_and_id =
        BtcBlockAndId::from_json(&BtcSubmissionMaterialJson::from_str(&read_to_string(s).unwrap()).unwrap()).unwrap();
    let output_index = 0;
    let tx = block_and_id.block.txdata[50].clone();
    let nonce = 1_584_612_094;
    let btc_deposit_address = "2NB1SRPSujETy9zYbXRZAGkk1zuDZHFAtyk".to_string();
    let eth_address = "provabletest".to_string();
    let eth_address_and_nonce_hash = "1729dce0b4e54e39610a95376a8bc96335fd93da68ae6aa27a62d4c282fb1ad3".to_string();
    let version = Some("1".to_string());
    let user_data = vec![];
    let chain_id_hex = None;
    let deposit_info_json = DepositAddressInfoJson::new(
        nonce,
        eth_address,
        btc_deposit_address,
        eth_address_and_nonce_hash,
        version,
        &user_data,
        chain_id_hex,
    )
    .unwrap();
    BtcUtxoAndValue::new(
        tx.output[output_index].value,
        &create_unsigned_utxo_from_tx(&tx, output_index as u32),
        Some(deposit_info_json),
        None,
    )
}

pub fn get_sample_p2sh_utxo_and_value_3() -> BtcUtxoAndValue {
    let s = "src/test_utils/btc-1670534-block-and-txs.json";
    let block_and_id =
        BtcBlockAndId::from_json(&BtcSubmissionMaterialJson::from_str(&read_to_string(s).unwrap()).unwrap()).unwrap();
    let output_index = 0;
    let tx = block_and_id.block.txdata[95].clone();
    let nonce = 1_584_696_514;
    let btc_deposit_address = "2My5iu4S78DRiH9capTKo8sXEa98yHVkQXg".to_string();
    let eth_address = "provabletest".to_string();
    let eth_address_and_nonce_hash = "d11539e07a521c78c20381c98cc546e3ccdd8a5c97f1cffe83ae5537f61a6e39".to_string();
    let version = Some("1".to_string());
    let user_data = vec![];
    let chain_id_hex = None;
    let deposit_info_json = DepositAddressInfoJson::new(
        nonce,
        eth_address,
        btc_deposit_address,
        eth_address_and_nonce_hash,
        version,
        &user_data,
        chain_id_hex,
    )
    .unwrap();
    BtcUtxoAndValue::new(
        tx.output[output_index].value,
        &create_unsigned_utxo_from_tx(&tx, output_index as u32),
        Some(deposit_info_json),
        None,
    )
}
