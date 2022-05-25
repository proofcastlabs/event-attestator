#[allow(dead_code)] // FIXME rm!
use std::fmt::Display;

use derive_more::Deref;
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::{
    chains::eth::eth_utils::convert_hex_to_eth_address,
    types::{Byte, Bytes, Result},
};

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize, Deref)]
pub struct DebugSignatories(Vec<DebugSignatory>);

impl DebugSignatories {
    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        serde_json::from_slice::<Vec<Bytes>>(bytes)?
            .iter()
            .map(|bytes| DebugSignatory::from_bytes(bytes))
            .collect::<Result<Vec<DebugSignatory>>>()
            .map(Self)
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(
            &self
                .iter()
                .map(|debug_signatory| debug_signatory.to_bytes())
                .collect::<Result<Vec<Bytes>>>()?,
        )?)
    }

    pub fn to_jsons(&self) -> Vec<DebugSignatoryJson> {
        self.iter().map(|signatory| signatory.to_json()).collect()
    }
}

impl Display for DebugSignatories {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self.to_jsons()).unwrap())
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebugSignatory {
    pub eth_address: EthAddress,
    pub nonce: u64,
}

impl Display for DebugSignatory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_json())
    }
}

impl DebugSignatory {
    pub fn from_json(json: &DebugSignatoryJson) -> Result<Self> {
        Ok(Self {
            nonce: json.nonce,
            eth_address: convert_hex_to_eth_address(&json.eth_address)?,
        })
    }

    pub fn to_json(self) -> DebugSignatoryJson {
        DebugSignatoryJson {
            nonce: self.nonce,
            eth_address: format!("0x{}", hex::encode(self.eth_address)),
        }
    }

    pub fn to_bytes(self) -> Result<Bytes> {
        self.to_json().to_bytes()
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        DebugSignatoryJson::from_bytes(bytes).and_then(|json| Self::from_json(&json))
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebugSignatoryJson {
    pub eth_address: String,
    pub nonce: u64,
}

impl DebugSignatoryJson {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}

impl Display for DebugSignatoryJson {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use rand::prelude::*;

    use super::*;

    fn get_random_debug_signatory() -> DebugSignatory {
        DebugSignatory {
            nonce: rand::thread_rng().gen(),
            eth_address: EthAddress::random(),
        }
    }

    fn get_n_random_debug_signatories(n: usize) -> DebugSignatories {
        DebugSignatories(
            vec![0; n]
                .iter()
                .map(|_| get_random_debug_signatory())
                .collect::<Vec<DebugSignatory>>(),
        )
    }

    #[test]
    fn should_serde_debug_signatories_to_bytes_and_back() {
        let signatories = get_n_random_debug_signatories(5);
        let bytes = signatories.to_bytes().unwrap();
        let result = DebugSignatories::from_bytes(&bytes).unwrap();
        assert_eq!(result, signatories)
    }
}
