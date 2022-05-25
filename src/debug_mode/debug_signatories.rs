#![allow(dead_code)] // FIXME rm!
use std::fmt::Display;

use derive_more::{Constructor, Deref};
use ethereum_types::Address as EthAddress;
use serde::{Deserialize, Serialize};

use crate::{
    chains::eth::eth_utils::convert_hex_to_eth_address,
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
};

lazy_static! {
    static ref DEBUG_SIGNATORIES_DB_KEY: [u8; 32] = crate::utils::get_prefixed_db_key("debug_signatories_db_key");
}

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
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

    pub fn get_from_db<D: DatabaseInterface>(db: &D) -> Result<Self> {
        match db.get(DEBUG_SIGNATORIES_DB_KEY.to_vec(), MIN_DATA_SENSITIVITY_LEVEL) {
            Ok(bytes) => Self::from_bytes(&bytes),
            Err(_) => Ok(Self::new(vec![])),
        }
    }

    pub fn put_in_db<D: DatabaseInterface>(&self, db: &D) -> Result<()> {
        db.put(
            DEBUG_SIGNATORIES_DB_KEY.to_vec(),
            self.to_bytes()?,
            MIN_DATA_SENSITIVITY_LEVEL,
        )
    }

    fn add(&self, signatory: &DebugSignatory) -> Self {
        let mut mutable_self = self.0.clone();
        if !mutable_self.contains(signatory) {
            mutable_self.push(*signatory);
        }
        Self(mutable_self)
    }

    fn remove(&self, eth_address: &EthAddress) -> Self {
        Self(
            self.iter()
                .filter(|signatory| signatory.eth_address != *eth_address)
                .cloned()
                .collect(),
        )
    }

    pub fn add_and_update_in_db<D: DatabaseInterface>(db: &D, signatory: &DebugSignatory) -> Result<()> {
        Self::get_from_db(db)
            .map(|signatories| signatories.add(&signatory))
            .and_then(|signatories| signatories.put_in_db(db))
    }

    pub fn remove_and_update_in_db<D: DatabaseInterface>(db: &D, eth_address: &EthAddress) -> Result<()> {
        Self::get_from_db(db)
            .map(|signatories| signatories.remove(eth_address))
            .and_then(|signatories| signatories.put_in_db(db))
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
    use crate::test_utils::get_test_database;

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

    #[test]
    fn should_put_and_get_signatories_from_db() {
        let db = get_test_database();
        let signatories = get_n_random_debug_signatories(5);
        signatories.put_in_db(&db).unwrap();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(result, signatories);
    }

    #[test]
    fn get_from_db_should_return_empty_signatories_if_none_in_db() {
        let db = get_test_database();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        let expected_result = DebugSignatories::new(vec![]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_add_and_put_in_db() {
        let db = get_test_database();
        let signatory = get_random_debug_signatory();
        let expected_result = DebugSignatories(vec![signatory.clone()]);
        DebugSignatories::add_and_update_in_db(&db, &signatory).unwrap();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_remove_and_update_in_db() {
        let db = get_test_database();
        let signatories = get_n_random_debug_signatories(5);
        signatories.put_in_db(&db).unwrap();
        let signatory_to_remove = signatories[2].eth_address.clone();
        let expected_result = DebugSignatories(
            signatories
                .iter()
                .filter(|signatory| signatory.eth_address != signatory_to_remove)
                .cloned()
                .collect(),
        );
        DebugSignatories::remove_and_update_in_db(&db, &signatory_to_remove).unwrap();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }
}
