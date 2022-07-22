#![allow(dead_code)] // FIXME rm!
use std::fmt::Display;

use ethereum_types::{Address as EthAddress, H256};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::{
    chains::eth::eth_utils::{convert_eth_address_to_string, convert_hex_to_eth_address},
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    crypto_utils::keccak_hash_bytes,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::strip_hex_prefix,
};

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebugSignatory {
    pub nonce: u64,
    pub name: String,
    pub eth_address: EthAddress,
}

impl DebugSignatory {
    pub fn new(name: &str, address: &EthAddress) -> Self {
        Self {
            nonce: 0,
            name: name.to_string(),
            eth_address: *address,
        }
    }

    pub fn increment_nonce(&self) -> Self {
        let mut mutable_self = self.clone();
        mutable_self.nonce = self.nonce + 1;
        mutable_self
    }

    pub fn to_json(self, debug_comannd_hash: &H256) -> Result<JsonValue> {
        Ok(json!({
            "nonce": self.nonce,
            "name": self.name.clone(),
            "eth_address": convert_eth_address_to_string(&self.eth_address),
            "hash_to_sign": self.hash_to_hex(&debug_comannd_hash)?,

        }))
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice::<Self>(&bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_mode::debug_signatures::test_utils::{get_sample_debug_command_hash, get_sample_debug_signatory};

    #[test]
    fn should_serde_debug_signatory_to_and_from_bytes() {
        let debug_signatory = get_sample_debug_signatory();
        let bytes = debug_signatory.to_bytes().unwrap();
        let result = DebugSignatory::from_bytes(&bytes).unwrap();
        assert_eq!(result, debug_signatory);
    }

    #[test]
    fn should_convert_debug_signatory_to_json() {
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let result = debug_signatory.to_json(&debug_command_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn should_increment_debug_signatory_nonce() {
        let debug_signatory = get_sample_debug_signatory();
        assert_eq!(debug_signatory.nonce, 0);
        let result = debug_signatory.increment_nonce();
        assert_eq!(result.nonce, 1);
    }
}
