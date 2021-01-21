#![cfg(test)]
use crate::chains::eos::eos_eth_token_dictionary::{EosEthTokenDictionary, EosEthTokenDictionaryEntry};

pub fn get_sample_eos_eth_token_dictionary() -> EosEthTokenDictionary {
    EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::from_str(&
    "{\"eos_token_decimals\":4,\"eth_token_decimals\":18,\"eos_symbol\":\"EOS\",\"eth_symbol\":\"PEOS\",\"eos_address\":\"eosio.token\",\"eth_address\":\"711c50b31ee0b9e8ed4d434819ac20b4fbbb5532\"}").unwrap()])
}
