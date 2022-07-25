#![allow(dead_code)] // FIXME rm!
use std::{fmt::Display, str::FromStr};

use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use web3::signing::recover;

use crate::{
    chains::eth::{eth_crypto::eth_signature::EthSignature, eth_utils::convert_hex_to_eth_address},
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    crypto_utils::keccak_hash_bytes,
    debug_mode::debug_signatures::debug_signatory::DebugSignatory,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::strip_hex_prefix,
};

/* FIXME TODO!
/// Debug Add Debug Signatory
///
/// Adds a debug signatory to the list. Requires a valid signature from that signer over the nonce
/// 0. If the signer is extant, nothing is changed.
pub fn debug_add_debug_signatory<D: DatabaseInterface>(
    db: &D,
    eth_address: &str,
    signature: &str,
) -> Result<String> {
    check_debug_mode()
        //.and_then(|_| DebugSignatories::get_from_db(db))
        // .and_then(|debug_signatories| )
        .and_then(|_| json!({"debug_add_signatory_success":true, "eth_address": eth_address}))
}

/// Debug Add Debug Signatory
///
/// Removes a debug signatory from the list. Requires a valid signature from an existing debug
/// signatory in order to do so. If the supplied eth address is not in the list of debug
/// debug_signatories, nothing is removed.
pub fn debug_remove_debug_signatory<D: DatabaseInterface>(
    db: &D,
    eth_address: &str,
    signature: &str,
) -> Result<String> {
    check_debug_mode()
        .and_then(|_| DebugSignatories::get_from_db(db))
        // .and_then(|debug_signatories| )
        .and_then(|_| json!({"debug_remove_signatory_success":true, "eth_address": eth_address}))
}
*/
lazy_static! {
    static ref DEBUG_SIGNATORIES_DB_KEY: [u8; 32] = crate::utils::get_prefixed_db_key("debug_signatories_db_key");
}

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize, Deref, Constructor)]
pub struct DebugSignatories(Vec<DebugSignatory>);

impl DebugSignatories {
    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        serde_json::from_slice::<Vec<Bytes>>(bytes)?
            .iter()
            .map(|bytes| DebugSignatory::from_bytes(bytes))
            .collect::<Result<Vec<DebugSignatory>>>()
            .map(Self)
    }

    fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(
            &self
                .iter()
                .cloned()
                .map(|debug_signatory| debug_signatory.to_bytes())
                .collect::<Result<Vec<Bytes>>>()?,
        )?)
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

    pub fn to_json(&self, debug_command_hash: &H256) -> Result<JsonValue> {
        Ok(json!([self
            .iter()
            .cloned()
            .map(|debug_signatory| debug_signatory.to_json(&debug_command_hash))
            .collect::<Result<Vec<_>>>()?]))
    }

    fn add(&self, debug_signatory: &DebugSignatory) -> Self {
        info!("✔ Maybe adding debug signatory to list...");
        let mut mutable_self = self.0.clone();
        let eth_address = debug_signatory.eth_address;
        match self.get(&eth_address) {
            Ok(signatory) => {
                warn!("✘ Debug signatory with ETH address '{}' already in list!", eth_address);
                Self(mutable_self)
            },
            Err(_) => {
                mutable_self.push(debug_signatory.clone());
                Self(mutable_self)
            },
        }
    }

    fn remove(&self, eth_address: &EthAddress) -> Self {
        Self(
            self.iter()
                .filter(|debug_signatory| debug_signatory.eth_address != *eth_address)
                .cloned()
                .collect(),
        )
    }

    fn get(&self, eth_address: &EthAddress) -> Result<DebugSignatory> {
        let signatories = self
            .iter()
            .filter(|debug_signatory| debug_signatory.eth_address == *eth_address)
            .cloned()
            .collect::<Vec<DebugSignatory>>();
        if signatories.is_empty() {
            Err(format!("Could not find debug signatory with address: '{}'!", eth_address).into())
        } else if signatories.len() > 1 {
            Err(format!("> 1 entry found with address: '{}'!", eth_address).into())
        } else {
            Ok(signatories[0].clone())
        }
    }

    fn replace(&self, debug_signatory: &DebugSignatory) -> Result<Self> {
        let eth_address = debug_signatory.eth_address;
        if self.get(&eth_address).is_ok() {
            Ok(self.remove(&eth_address).add(debug_signatory))
        } else {
            Err(format!("Cannot replace entry, none exists with eth address: '{}'!", eth_address).into())
        }
    }

    pub fn add_and_update_in_db<D: DatabaseInterface>(db: &D, debug_signatory: &DebugSignatory) -> Result<()> {
        Self::get_from_db(db)
            .map(|debug_signatories| debug_signatories.add(debug_signatory))
            .and_then(|debug_signatories| debug_signatories.put_in_db(db))
    }

    pub fn remove_and_update_in_db<D: DatabaseInterface>(db: &D, eth_address: &EthAddress) -> Result<()> {
        Self::get_from_db(db)
            .map(|debug_signatories| debug_signatories.remove(eth_address))
            .and_then(|debug_signatories| debug_signatories.put_in_db(db))
    }

    fn increment_nonce_in_signatory_in_db<D: DatabaseInterface>(db: &D, eth_address: &EthAddress) -> Result<()> {
        let debug_signatories = Self::get_from_db(db)?;
        debug_signatories
            .get(eth_address)
            .map(|signatory| signatory.increment_nonce())
            .and_then(|signatory| debug_signatories.replace(&signatory))
            .and_then(|debug_signatories| debug_signatories.put_in_db(db))
    }

    fn maybe_validate_signature_for_eth_address_and_increment_nonce_in_db<D: DatabaseInterface>(
        db: &D,
        eth_address: &EthAddress,
        debug_command_hash: &H256,
        signature_str: &str,
    ) -> Result<()> {
        let debug_signatories = Self::get_from_db(db)?;
        debug_signatories
            .get(eth_address)
            .and_then(|signatory| signatory.validate(&EthSignature::from_str(signature_str)?, &debug_command_hash))
            .and_then(|_| Self::increment_nonce_in_signatory_in_db(db, eth_address))
    }

    fn to_eth_addresses(&self) -> Vec<EthAddress> {
        self.iter().map(|signatory| signatory.eth_address).collect::<Vec<_>>()
    }

    pub fn maybe_validate_signature_and_increment_nonce_in_db<D: DatabaseInterface>(
        db: &D,
        debug_command_hash: &H256,
        signature_str: &str,
    ) -> Result<()> {
        Self::get_from_db(db)
            .map(|debug_signatories| debug_signatories.to_eth_addresses())
            .and_then(|eth_addresses| {
                if eth_addresses
                    .iter()
                    .filter_map(|eth_address| {
                        match Self::maybe_validate_signature_for_eth_address_and_increment_nonce_in_db(
                            db,
                            eth_address,
                            debug_command_hash,
                            signature_str,
                        ) {
                            Ok(_) => Some(true),
                            Err(_) => None,
                        }
                    })
                    .next()
                    .is_none()
                {
                    Err("Signature not valid for any debug signatories!".into())
                } else {
                    Ok(())
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        debug_mode::debug_signatures::test_utils::{
            get_n_random_debug_signatories,
            get_sample_debug_command_hash,
            get_sample_debug_signatories,
            get_sample_private_key,
        },
        errors::AppError,
        test_utils::get_test_database,
    };

    #[test]
    fn should_serde_debug_signatories_to_and_from_bytes() {
        let debug_signatories = get_sample_debug_signatories();
        let bytes = debug_signatories.to_bytes().unwrap();
        let result = DebugSignatories::from_bytes(&bytes).unwrap();
        assert_eq!(result, debug_signatories);
    }

    #[test]
    fn should_put_and_get_debug_signatories_in_and_from_db() {
        let debug_signatories = get_sample_debug_signatories();
        let db = get_test_database();
        debug_signatories.put_in_db(&db).unwrap();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(result, debug_signatories);
    }

    #[test]
    fn get_from_db_should_return_empty_signatories_if_none_in_db() {
        let db = get_test_database();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        let expected_result = DebugSignatories::new(vec![]);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_convert_debug_signatories_to_json() {
        let debug_signatories = get_sample_debug_signatories();
        let debug_command_hash = get_sample_debug_command_hash();
        let result = debug_signatories.to_json(&debug_command_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn should_add_debug_signatory() {
        let debug_signatories = get_sample_debug_signatories();
        let debug_signatory = DebugSignatory::default();
        assert!(!debug_signatories.contains(&debug_signatory));
        let result = debug_signatories.add(&debug_signatory);
        assert!(result.contains(&debug_signatory));
    }

    #[test]
    fn should_not_add_debug_signatory_if_extant() {
        let debug_signatory = DebugSignatory::random();
        let debug_signatories = DebugSignatories::new(vec![debug_signatory.clone()]);
        assert_eq!(debug_signatories.len(), 1);
        let result = debug_signatories.add(&debug_signatory);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn should_remove_debug_signatory() {
        let debug_signatories = get_sample_debug_signatories();
        let debug_signatory = debug_signatories[1].clone();
        assert!(debug_signatories.contains(&debug_signatory));
        let result = debug_signatories.remove(&debug_signatory.eth_address);
        assert!(!result.contains(&debug_signatory));
    }

    #[test]
    fn should_get_debug_signatory_from_signatories() {
        let debug_signatories = get_sample_debug_signatories();
        let expected_result = debug_signatories[2].clone();
        let eth_address = expected_result.eth_address.clone();
        let result = debug_signatories.get(&eth_address).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_get_non_extant_debug_signatory_from_signatories() {
        let debug_signatories = get_sample_debug_signatories();
        let eth_address = EthAddress::random();
        let expected_error = format!("Could not find debug signatory with address: '{}'!", eth_address);
        match debug_signatories.get(&eth_address) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_fail_to_replace_non_existent_entry() {
        let debug_signatories = get_sample_debug_signatories();
        let debug_signatory = DebugSignatory::random();
        let expected_error = format!(
            "Cannot replace entry, none exists with eth address: '{}'!",
            debug_signatory.eth_address
        );
        match debug_signatories.replace(&debug_signatory) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_replace_entry() {
        let debug_signatories = get_sample_debug_signatories();
        let index = 1;
        let mut debug_signatory = debug_signatories[index].clone();
        let eth_address = debug_signatory.eth_address;
        debug_signatory.nonce = debug_signatory.nonce + 1;
        assert_ne!(debug_signatory, debug_signatories[index]);
        let updated_signatories = debug_signatories.replace(&debug_signatory).unwrap();
        let result = updated_signatories.get(&eth_address).unwrap();
        assert_eq!(result, debug_signatory);
    }

    #[test]
    fn should_add_and_update_in_db() {
        let db = get_test_database();
        let debug_signatory = DebugSignatory::random();
        let expected_result = DebugSignatories(vec![debug_signatory.clone()]);
        DebugSignatories::add_and_update_in_db(&db, &debug_signatory).unwrap();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_remove_and_update_in_db() {
        let db = get_test_database();
        let debug_signatories = get_n_random_debug_signatories(5);
        debug_signatories.put_in_db(&db).unwrap();
        let signatory_to_remove = debug_signatories[2].eth_address.clone();
        let expected_result = DebugSignatories(
            debug_signatories
                .iter()
                .filter(|debug_signatory| debug_signatory.eth_address != signatory_to_remove)
                .cloned()
                .collect(),
        );
        DebugSignatories::remove_and_update_in_db(&db, &signatory_to_remove).unwrap();
        let result = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_increment_nonce_in_entry_in_db() {
        let debug_signatories = get_n_random_debug_signatories(5);
        let db = get_test_database();
        let index = 2;
        let debug_signatory = debug_signatories[index].clone();
        let eth_address = debug_signatory.eth_address;
        let nonce_before = debug_signatory.nonce;
        debug_signatories.put_in_db(&db).unwrap();
        DebugSignatories::increment_nonce_in_signatory_in_db(&db, &eth_address).unwrap();
        let updated_signatories = DebugSignatories::get_from_db(&db).unwrap();
        let expected_result = nonce_before + 1;
        let result = updated_signatories.get(&eth_address).unwrap().nonce;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_error_if_entry_in_signatories_twice() {
        let debug_signatory = DebugSignatory::random();
        // NOTE: This is the only way we can create one with a duplicate in it.
        let debug_signatories = DebugSignatories(vec![debug_signatory.clone(), debug_signatory.clone()]);
        let eth_address = debug_signatory.eth_address.clone();
        let expected_error = format!("> 1 entry found with address: '{}'!", eth_address);
        match debug_signatories.get(&eth_address) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_maybe_validate_debug_signature_and_increment_nonce_in_db() {
        let db = get_test_database();
        let eth_address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let debug_command_hash = H256::random();
        let debug_signatory_1 = DebugSignatory::new("Some name", &eth_address);
        let debug_signatory_2 = DebugSignatory::random();
        let debug_signatory_3 = DebugSignatory::random();

        // NOTE: Assert the signatory we care about's nonce is 0.
        assert_eq!(debug_signatory_1.nonce, 0);
        let debug_signatories = DebugSignatories::new(vec![
            debug_signatory_2.clone(),
            debug_signatory_1.clone(),
            debug_signatory_3.clone(),
        ]);
        debug_signatories.put_in_db(&db).unwrap();
        let pk = get_sample_private_key();
        // NOTE: Assert the private key is for the address we expect.
        assert_eq!(pk.to_public_key().to_address(), eth_address);

        // NOTE Now we sign the random `debug_command_hash`
        let signature = debug_signatory_1.sign(&pk, &debug_command_hash).unwrap().to_string();

        // NOTE: Signature should be valid, and the nonce for this signatory should be incremented.
        DebugSignatories::maybe_validate_signature_and_increment_nonce_in_db(&db, &debug_command_hash, &signature)
            .unwrap();

        // NOTE: So lets assert that this signatory's nonce did indeed get updated in the db.
        let updated_signatories = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(updated_signatories.get(&eth_address).unwrap().nonce, 1);

        // NOTE: And finally, assert that the other two signatory's nonces did NOT get
        // incremented.
        assert_eq!(
            updated_signatories.get(&debug_signatory_2.eth_address).unwrap().nonce,
            debug_signatory_2.nonce
        );
        assert_eq!(
            updated_signatories.get(&debug_signatory_3.eth_address).unwrap().nonce,
            debug_signatory_3.nonce
        );
    }

    #[test]
    fn should_fail_to_maybe_validate_invalid_signature_and_thus_not_increment_nonce_in_db() {
        let db = get_test_database();
        let debug_signatories_before = get_sample_debug_signatories();
        debug_signatories_before.put_in_db(&db).unwrap();
        let debug_command_hash = get_sample_debug_command_hash();

        // NOTE: The signature is totally random...
        let random_signature = EthSignature::random().unwrap().to_string();
        // NOTE: And so it should error...
        let expected_error = "Signature not valid for any debug signatories!";
        match DebugSignatories::maybe_validate_signature_and_increment_nonce_in_db(
            &db,
            &debug_command_hash,
            &random_signature,
        ) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }

        // NOTE: Finally, let's assert that no debug signatory nonces were incremented.
        let debug_signatories_after = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(debug_signatories_before, debug_signatories_after);
    }
}
