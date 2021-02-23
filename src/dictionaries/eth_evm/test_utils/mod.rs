use ethereum_types::Address as EthAddress;

use crate::dictionaries::eth_evm::{EthEvmTokenDictionary, EthEvmTokenDictionaryEntry, EthEvmTokenDictionaryJson};

pub fn get_sample_eth_evm_token_dictionary_entry_1() -> EthEvmTokenDictionaryEntry {
    EthEvmTokenDictionaryEntry::new(
        "token1".to_string(),
        "pToken1".to_string(),
        EthAddress::from_slice(&hex::decode("9f57CB2a4F462a5258a49E88B4331068a391DE66").unwrap()),
        EthAddress::from_slice(&hex::decode("45a36a8e118C37e4c47eF4Ab827A7C9e579E11E2").unwrap()),
    )
}

pub fn get_sample_eth_evm_token_dictionary_entry_2() -> EthEvmTokenDictionaryEntry {
    EthEvmTokenDictionaryEntry::new(
        "token2".to_string(),
        "pToken2".to_string(),
        EthAddress::from_slice(&hex::decode("45a36a8e118c37e4c47ef4ab827a7c9e579e11e2").unwrap()),
        EthAddress::from_slice(&hex::decode("6b50911190cffd34b50d575706639c24aafdc625").unwrap()),
    )
}

pub fn get_sample_eth_evm_token_dictionary() -> EthEvmTokenDictionary {
    EthEvmTokenDictionary::new(vec![
        get_sample_eth_evm_token_dictionary_entry_1(),
        get_sample_eth_evm_token_dictionary_entry_2(),
    ])
}

pub fn get_sample_eth_evm_token_dictionary_json() -> EthEvmTokenDictionaryJson {
    get_sample_eth_evm_token_dictionary().to_json().unwrap()
}
