#![cfg(test)]
use std::{fs::read_to_string, path::Path, str::FromStr};

use common::{
    dictionaries::eth_evm::{EthEvmTokenDictionary, EthEvmTokenDictionaryEntry},
    types::Result,
    EthChainId,
};
use common_eth::{convert_hex_to_eth_address, get_random_eth_address, EthPrivateKey, EthSubmissionMaterial};
use ethereum_types::Address as EthAddress;
use serde_json::json;

use crate::{
    eth::{Erc20OnIntIntTxInfo, Erc20OnIntIntTxInfos},
    int::{Erc20OnIntEthTxInfo, Erc20OnIntEthTxInfos},
};

pub fn get_sample_eth_tx_infos() -> Erc20OnIntEthTxInfos {
    let dictionary = get_sample_token_dictionary();
    let material = get_sample_peg_out_submission_material();
    let origin_chain_id = EthChainId::Ropsten;
    let vault_address = get_random_eth_address();
    let router_address = get_random_eth_address();
    Erc20OnIntEthTxInfos::from_submission_material(
        &material,
        &dictionary,
        &origin_chain_id,
        &vault_address,
        &router_address,
    )
    .unwrap()
}

pub fn get_sample_eth_tx_info() -> Erc20OnIntEthTxInfo {
    get_sample_eth_tx_infos()[0].clone()
}

pub fn get_sample_int_tx_infos() -> Erc20OnIntIntTxInfos {
    let material = get_sample_peg_in_1_submission_material();
    let vault_address = get_sample_vault_address();
    let dictionary = get_sample_token_dictionary();
    let router_address = get_sample_router_address();
    Erc20OnIntIntTxInfos::from_submission_material(&material, &vault_address, &dictionary, &router_address).unwrap()
}

pub fn get_sample_int_tx_info() -> Erc20OnIntIntTxInfo {
    get_sample_int_tx_infos()[0].clone()
}

pub fn get_sample_peg_in_1_submission_material() -> EthSubmissionMaterial {
    EthSubmissionMaterial::from_str(&read_to_string("src/test_utils/eth-peg-in-block-1.json").unwrap()).unwrap()
}

pub fn get_sample_peg_in_1_submission_string() -> String {
    read_to_string("src/test_utils/eth-peg-in-block-1.json").unwrap()
}

pub fn get_sample_goerli_peg_in_submission_string() -> String {
    read_to_string("src/test_utils/goerli-peg-in-block.json").unwrap()
}

pub fn get_sample_sepolia_peg_in_submission_string() -> String {
    read_to_string("src/test_utils/sepolia-peg-in-block.json").unwrap()
}

pub fn get_sample_token_dictionary() -> EthEvmTokenDictionary {
    EthEvmTokenDictionary::new(vec![get_sample_token_dictionary_entry()])
}

pub fn get_sample_router_address() -> EthAddress {
    convert_hex_to_eth_address("0x0e1c8524b1d1891b201ffc7bb58a82c96f8fc4f6").unwrap()
}

pub fn get_sample_token_dictionary_entry() -> EthEvmTokenDictionaryEntry {
    EthEvmTokenDictionaryEntry::from_str(
        &json!({
            "eth_symbol":"PNT",
            "evm_symbol":"tiPNT",
            "evm_address":"0xa83446f219baec0b6fd6b3031c5a49a54543045b",
            "eth_address":"0xc63ab9437f5589e2c67e04c00a98506b43127645",
            "eth_fee_basis_points":10,
            "evm_fee_basis_points":25,
            "eth_token_decimals":18,
            "evm_token_decimals":18
        })
        .to_string(),
    )
    .unwrap()
}

pub fn get_sample_goerli_token_dictionary_entry() -> EthEvmTokenDictionaryEntry {
    EthEvmTokenDictionaryEntry::from_str(
        &json!({
            "eth_symbol":"PNT",
            "evm_symbol":"tiPNT",
            "evm_address":"0xa83446f219baec0b6fd6b3031c5a49a54543045b",
            "eth_address":"0x5eb802aBE474290aacC1EF2786431E1FF6C03191",
            "eth_fee_basis_points":10,
            "evm_fee_basis_points":25,
            "eth_token_decimals":18,
            "evm_token_decimals":18
        })
        .to_string(),
    )
    .unwrap()
}

pub fn get_sample_sepolia_token_dictionary_entry() -> EthEvmTokenDictionaryEntry {
    EthEvmTokenDictionaryEntry::from_str(
        &json!({
            "eth_symbol":"PNT",
            "evm_symbol":"tiPNT",
            "evm_address":"0xa83446f219baec0b6fd6b3031c5a49a54543045b",
            "eth_address":"0x5eb802aBE474290aacC1EF2786431E1FF6C03191",
            "eth_fee_basis_points":10,
            "evm_fee_basis_points":25,
            "eth_token_decimals":18,
            "evm_token_decimals":18
        })
        .to_string(),
    )
    .unwrap()
}

fn get_sample_submission_material_string_n(chain_type: &str, n: usize) -> Result<String> {
    let path = format!("src/test_utils/{}-submission-material-{}.json", chain_type, n);
    match Path::new(&path).exists() {
        true => Ok(read_to_string(path)?),
        false => Err(format!(
            "✘ Cannot find sample {} submission material #{} file!",
            chain_type.to_uppercase(),
            n
        )
        .into()),
    }
}

pub fn get_evm_submission_material_n(n: usize) -> EthSubmissionMaterial {
    EthSubmissionMaterial::from_str(&get_sample_submission_material_string_n("evm", n).unwrap()).unwrap()
}

pub fn get_eth_submission_material_n(n: usize) -> EthSubmissionMaterial {
    EthSubmissionMaterial::from_str(&get_sample_submission_material_string_n("eth", n).unwrap()).unwrap()
}

const ERC20_ON_EVM_DICTIONARY_JSON: &str = "[{\"eth_symbol\":\"PNT\",\"evm_symbol\":\"PNT\",\"evm_address\":\"0xdaacb0ab6fb34d24e8a67bfa14bf4d95d4c7af92\",\"eth_address\":\"0x89ab32156e46f46d02ade3fecbe5fc4243b9aaed\"},{\"eth_symbol\":\"OPIUM\",\"evm_symbol\":\"pOPIUM\",\"evm_address\":\"0x566cedd201f67e542a6851a2959c1a449a041945\",\"eth_address\":\"0x888888888889c00c67689029d7856aac1065ec11\"},{\"eth_symbol\":\"PTERIA\",\"evm_symbol\":\"PTERIA\",\"evm_address\":\"0x9f5377fa03dcd4016a33669b385be4d0e02f27bc\",\"eth_address\":\"0x02eca910cb3a7d43ebc7e8028652ed5c6b70259b\"},{\"eth_symbol\":\"BCP\",\"evm_symbol\":\"pBCP\",\"evm_address\":\"0xa114f89b49d6a58416bb07dbe09502c4f3a19e2f\",\"eth_address\":\"0xe4f726adc8e89c6a6017f01eada77865db22da14\"},{\"eth_symbol\":\"DEFI++\",\"evm_symbol\":\"pDEFI++\",\"evm_address\":\"0xae22e27d1f727b585549c10e26192b2bc01082ca\",\"eth_address\":\"0x8d1ce361eb68e9e05573443c407d4a3bed23b033\"}]";

pub fn get_sample_eth_evm_token_dictionary() -> EthEvmTokenDictionary {
    EthEvmTokenDictionary::from_str(ERC20_ON_EVM_DICTIONARY_JSON).unwrap()
}

pub fn get_sample_vault_address() -> EthAddress {
    convert_hex_to_eth_address("0x866e3fc7043efb8ff3a994f7d59f53fe045d4d7a").unwrap()
}

pub fn get_sample_eth_private_key() -> EthPrivateKey {
    EthPrivateKey::from_slice(&hex::decode("115bfcb3fd7cae5c2b996bf7bd1c012f804b98060f7e2f4d558542549e88440f").unwrap())
        .unwrap()
}

pub fn get_sample_evm_private_key() -> EthPrivateKey {
    EthPrivateKey::from_slice(&hex::decode("57a5a09577a0604b84870577598d4a24fe9e5b879650a0248ac96be7d9d3f3aa").unwrap())
        .unwrap()
}

pub fn get_sample_eth_init_block_json_string() -> String {
    read_to_string("src/test_utils/eth-core-init-block.json").unwrap()
}

pub fn get_sample_goerli_peg_in_init_block_json_string() -> String {
    read_to_string("src/test_utils/goerli-core-peg-in-init-block.json").unwrap()
}

pub fn get_sample_sepolia_init_block_json_string() -> String {
    read_to_string("src/test_utils/sepolia-init-block.json").unwrap()
}

pub fn get_sample_int_init_block_json_string() -> String {
    read_to_string("src/test_utils/int-core-init-block.json").unwrap()
}

pub fn get_sample_peg_out_json_string() -> String {
    read_to_string("src/test_utils/int-peg-out-block-1.json").unwrap()
}

pub fn get_sample_peg_out_submission_material() -> EthSubmissionMaterial {
    EthSubmissionMaterial::from_str(&get_sample_peg_out_json_string()).unwrap()
}

mod tests {
    use super::*;

    #[test]
    fn should_get_evm_submission_material_n() {
        get_evm_submission_material_n(1);
    }

    #[test]
    fn should_get_eth_submission_material_n() {
        get_eth_submission_material_n(1);
    }

    #[test]
    fn should_get_sample_eth_private_key() {
        get_sample_eth_private_key();
    }

    #[test]
    fn should_get_sample_evm_private_key() {
        get_sample_evm_private_key();
    }

    #[test]
    fn should_get_sample_eth_evm_dictionary() {
        get_sample_eth_evm_token_dictionary();
    }
}
