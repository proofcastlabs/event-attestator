#![cfg(test)]
use std::{fs::read_to_string, path::Path};

use ethereum_types::Address as EthAddress;

use crate::{
    chains::{
        eth::{eth_crypto::eth_private_key::EthPrivateKey, eth_submission_material::EthSubmissionMaterial},
        evm::{
            eth_crypto::eth_private_key::EthPrivateKey as EvmPrivateKey,
            eth_submission_material::EthSubmissionMaterial as EvmSubmissionMaterial,
        },
    },
    dictionaries::eth_evm::{EthEvmTokenDictionary, EthEvmTokenDictionaryEntry},
    types::Result,
};

fn get_sample_submission_material_string_n(chain_type: &str, n: usize) -> Result<String> {
    let path = format!(
        "src/eth_on_evm/test_utils/{}-submission-material-{}.json",
        chain_type, n
    );
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

pub fn get_evm_submission_material_n(n: usize) -> EvmSubmissionMaterial {
    EvmSubmissionMaterial::from_str(&get_sample_submission_material_string_n("evm", n).unwrap()).unwrap()
}

pub fn get_eth_submission_material_n(n: usize) -> EthSubmissionMaterial {
    EthSubmissionMaterial::from_str(&get_sample_submission_material_string_n("eth", n).unwrap()).unwrap()
}

pub fn get_sample_eth_evm_token_dictionary() -> EthEvmTokenDictionary {
    EthEvmTokenDictionary::new(vec![EthEvmTokenDictionaryEntry::from_str(&
        "{\"eth_symbol\":\"ERC\",\"evm_symbol\":\"pERC20\",\"evm_address\":\"0x6819bbfdf803b8b87850916d3eeb3642dde6c24f\",\"eth_address\":\"0xbf6ab900f1a3d8f94bc98f1d2ba1b8f00d532078\"}"
    ).unwrap()])
}

pub fn get_sample_vault_address() -> EthAddress {
    EthAddress::from_slice(&hex::decode("DAf39ecC934c69fc1035D4cb03f640630344bb42").unwrap())
}

pub fn get_sample_eth_private_key() -> EthPrivateKey {
    EthPrivateKey::from_slice(&hex::decode("115bfcb3fd7cae5c2b996bf7bd1c012f804b98060f7e2f4d558542549e88440f").unwrap())
        .unwrap()
}

pub fn get_sample_evm_private_key() -> EvmPrivateKey {
    EvmPrivateKey::from_slice(&hex::decode("57a5a09577a0604b84870577598d4a24fe9e5b879650a0248ac96be7d9d3f3aa").unwrap())
        .unwrap()
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
}
