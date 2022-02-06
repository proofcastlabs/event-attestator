use std::str::FromStr;

use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::{Address as EthAddress, U256};
use rust_algorand::AlgorandAddress;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    chains::{algo::algo_state::AlgoState, eth::eth_state::EthState},
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    dictionaries::{dictionary_constants::EVM_ALGO_DICTIONARY_KEY, dictionary_traits::DictionaryDecimalConverter},
    errors::AppError,
    fees::fee_utils::get_last_withdrawal_date_as_human_readable_string,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::{get_unix_timestamp, strip_hex_prefix},
};

#[derive(Default, Debug, Clone, Eq, PartialEq, Constructor, Deserialize, Serialize)]
pub struct EvmAlgoTokenDictionaryEntry {
    pub evm_token_decimals: u16,
    pub algo_token_decimals: u16,
    pub evm_address: EthAddress,
    pub algo_address: AlgorandAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, Constructor, Deserialize, Serialize)]
pub struct EvmAlgoTokenDictionaryEntryJson {
    pub evm_address: String,
    pub algo_address: String,
    pub evm_token_decimals: u16,
    pub algo_token_decimals: u16,
}

impl DictionaryDecimalConverter for EvmAlgoTokenDictionaryEntry {
    fn requires_decimal_conversion(&self) -> Result<bool> {
        Ok(self.get_host_decimals()? != self.get_native_decimals()?)
    }

    fn get_host_decimals(&self) -> Result<u16> {
        Ok(self.algo_token_decimals)
    }

    fn get_native_decimals(&self) -> Result<u16> {
        Ok(self.evm_token_decimals)
    }
}

impl EvmAlgoTokenDictionaryEntry {
    fn requires_decimal_conversion(&self) -> bool {
        self.algo_token_decimals != self.evm_token_decimals
    }

    fn to_json(&self) -> Result<EvmAlgoTokenDictionaryEntryJson> {
        Ok(EvmAlgoTokenDictionaryEntryJson {
            algo_address: self.algo_address.to_string(),
            evm_token_decimals: self.evm_token_decimals,
            algo_token_decimals: self.algo_token_decimals,
            evm_address: hex::encode(&self.evm_address.as_bytes()),
        })
    }

    pub fn from_json(json: &EvmAlgoTokenDictionaryEntryJson) -> Result<Self> {
        Ok(Self {
            algo_token_decimals: json.algo_token_decimals,
            evm_token_decimals: json.evm_token_decimals,
            algo_address: AlgorandAddress::from_str(&json.algo_address)?,
            evm_address: EthAddress::from_slice(&hex::decode(strip_hex_prefix(&json.evm_address))?),
        })
    }

    pub fn convert_evm_amount_to_algo_amount(&self, amount: U256) -> Result<U256> {
        info!("✔ Converting from EVM amount to ALGO amount...");
        self.convert_native_amount_to_host_amount(amount)
    }

    pub fn convert_algo_amount_to_evm_amount(&self, amount: U256) -> Result<U256> {
        info!("✔ Converting from ALGO amount to EVM amount...");
        self.convert_host_amount_to_native_amount(amount)
    }
}

impl FromStr for EvmAlgoTokenDictionaryEntryJson {
    type Err = AppError;

    fn from_str(s: &str) -> Result<Self> {
        Ok(serde_json::from_str(s)?)
    }
}

impl std::fmt::Display for EvmAlgoTokenDictionaryEntryJson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(self).to_string())
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
}
