#![allow(dead_code)] // FIXME rm!

use std::str::FromStr;

use derive_more::Constructor;
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    dictionaries::dictionary_traits::DictionaryDecimalConverter,
    errors::AppError,
    types::Result,
    utils::strip_hex_prefix,
};

#[derive(Default, Debug, Clone, Eq, PartialEq, Constructor, Deserialize, Serialize)]
pub struct EvmAlgoTokenDictionaryEntry {
    pub evm_decimals: u16,
    pub algo_decimals: u16,
    pub algo_asset_id: u64,
    pub evm_symbol: String,
    pub algo_symbol: String,
    pub evm_address: EthAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deserialize, Serialize)]
pub struct EvmAlgoTokenDictionaryEntryJson {
    pub evm_decimals: u16,
    pub algo_decimals: u16,
    pub algo_asset_id: u64,
    pub evm_symbol: String,
    pub algo_symbol: String,
    pub evm_address: String,
}

impl DictionaryDecimalConverter for EvmAlgoTokenDictionaryEntry {
    fn requires_decimal_conversion(&self) -> Result<bool> {
        Ok(self.get_host_decimals()? != self.get_native_decimals()?)
    }

    fn get_host_decimals(&self) -> Result<u16> {
        Ok(self.algo_decimals)
    }

    fn get_native_decimals(&self) -> Result<u16> {
        Ok(self.evm_decimals)
    }
}

impl EvmAlgoTokenDictionaryEntry {
    fn requires_decimal_conversion(&self) -> bool {
        self.algo_decimals != self.evm_decimals
    }

    pub fn to_json(&self) -> Result<EvmAlgoTokenDictionaryEntryJson> {
        Ok(EvmAlgoTokenDictionaryEntryJson {
            evm_decimals: self.evm_decimals,
            algo_decimals: self.algo_decimals,
            algo_asset_id: self.algo_asset_id,
            evm_symbol: self.evm_symbol.clone(),
            algo_symbol: self.algo_symbol.clone(),
            evm_address: hex::encode(self.evm_address.as_bytes()),
        })
    }

    pub fn from_json(json: &EvmAlgoTokenDictionaryEntryJson) -> Result<Self> {
        Ok(Self {
            evm_decimals: json.evm_decimals,
            algo_decimals: json.algo_decimals,
            algo_asset_id: json.algo_asset_id,
            evm_symbol: json.evm_symbol.clone(),
            algo_symbol: json.algo_symbol.clone(),
            evm_address: EthAddress::from_slice(&hex::decode(strip_hex_prefix(&json.evm_address))?),
        })
    }

    pub fn convert_evm_amount_to_algo_amount(&self, amount: U256) -> Result<U256> {
        info!("✔ Converting from EVM amount to ALGO amount...");
        self.convert_native_amount_to_host_amount(amount)
    }

    pub fn convert_algo_amount_to_evm_amount(&self, amount: u64) -> Result<U256> {
        info!("✔ Converting from ALGO amount to EVM amount...");
        self.convert_host_amount_to_native_amount(U256::from(amount))
    }
}

impl FromStr for EvmAlgoTokenDictionaryEntryJson {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

impl FromStr for EvmAlgoTokenDictionaryEntry {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Self::from_json(&serde_json::from_str::<EvmAlgoTokenDictionaryEntryJson>(s)?)
    }
}

impl std::fmt::Display for EvmAlgoTokenDictionaryEntryJson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serde_evm_algo_dictionary_entry_to_and_from_json() {
        let entry = EvmAlgoTokenDictionaryEntry::default();
        let json = entry.to_json().unwrap();
        let result = EvmAlgoTokenDictionaryEntry::from_json(&json).unwrap();
        assert_eq!(entry, result);
    }

    #[test]
    fn should_get_evm_algo_dictionary_entry_from_str() {
        let s = "{\"algo_symbol\":\"SYM\",\"evm_symbol\":\"SYM\",\"evm_decimals\": 18,\"algo_decimals\": 18,\"algo_asset_id\": 666,\"evm_address\": \"0xCE141c45619e9AdBDBdDA5af19B3052Ff79d5663\"}";
        let result = EvmAlgoTokenDictionaryEntry::from_str(s);
        assert!(result.is_ok());
    }
}
