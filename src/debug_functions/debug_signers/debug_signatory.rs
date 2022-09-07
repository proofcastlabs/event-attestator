use ethereum_types::{Address as EthAddress, H256};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};

use crate::{
    chains::eth::eth_utils::{convert_eth_address_to_string, convert_h256_to_string},
    core_type::CoreType,
    types::{Byte, Bytes, Result},
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
        info!(
            "✔ Incrementing nonce for '{}' from {} to {}!",
            self.name,
            self.nonce,
            self.nonce + 1
        );
        let mut mutable_self = self.clone();
        mutable_self.nonce = self.nonce + 1;
        mutable_self
    }

    pub fn to_json(&self, core_type: &CoreType, debug_command_hash: &H256) -> Result<JsonValue> {
        /*
         * NOTE: We use this so that parsing this output via `jq` or similar is as simple and quick
         * as possible for when this functionality is being used on mobile devices or similar.
         * NOTE: Glossary:
         *  n = nonce
         *  a = ethAddress
         *  h = hashToSign
         *  d = debugCommandHash
         */
        Ok(json!({
            "n": self.nonce,
            "h": self.hash_to_hex(core_type, debug_command_hash)?,
            "d": convert_h256_to_string(debug_command_hash),
            "a": self.eth_address(),

        }))
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice::<Self>(bytes)?)
    }

    pub fn to_enclave_state_json(&self) -> JsonValue {
        json!({
            "name": self.name,
            "nonce": self.nonce,
            "ethAddress": convert_eth_address_to_string(&self.eth_address),
        })
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn eth_address(&self) -> String {
        convert_eth_address_to_string(&self.eth_address)
    }
}

#[cfg(test)]
use rand::{
    distributions::{Alphanumeric, DistString},
    Rng,
};

#[cfg(test)]
impl DebugSignatory {
    pub fn random() -> Self {
        Self {
            nonce: rand::thread_rng().gen(),
            eth_address: EthAddress::random(),
            name: Alphanumeric.sample_string(&mut rand::thread_rng(), 8),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_functions::debug_signers::test_utils::{
        get_sample_debug_command_hash,
        get_sample_debug_signatory,
    };

    #[test]
    fn should_serde_debug_signatory_to_and_from_bytes() {
        let debug_signatory = get_sample_debug_signatory();
        let bytes = debug_signatory.to_bytes().unwrap();
        let result = DebugSignatory::from_bytes(&bytes).unwrap();
        assert_eq!(result, debug_signatory);
    }

    #[test]
    fn should_convert_debug_signatory_to_json() {
        let core_type = CoreType::BtcOnInt;
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let result = debug_signatory.to_json(&core_type, &debug_command_hash);
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
