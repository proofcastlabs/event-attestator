#![allow(dead_code)] // FIXME rm!
use std::fmt::Display;

use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256};
use serde::{Deserialize, Serialize};
use serde_json::json;
use web3::signing::recover;

use crate::{
    chains::eth::eth_utils::convert_hex_to_eth_address,
    constants::MIN_DATA_SENSITIVITY_LEVEL,
    crypto_utils::keccak_hash_bytes,
    debug_mode::debug_signatures::debug_signatory::{DebugSignatory, DebugSignatoryJson},
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
        // .and_then(|signatories| )
        .and_then(|_| json!({"debug_add_signatory_success":true, "eth_address": eth_address}))
}

/// Debug Add Debug Signatory
///
/// Removes a debug signatory from the list. Requires a valid signature from an existing debug
/// signatory in order to do so. If the supplied eth address is not in the list of debug
/// signatories, nothing is removed.
pub fn debug_remove_debug_signatory<D: DatabaseInterface>(
    db: &D,
    eth_address: &str,
    signature: &str,
) -> Result<String> {
    check_debug_mode()
        .and_then(|_| DebugSignatories::get_from_db(db))
        // .and_then(|signatories| )
        .and_then(|_| json!({"debug_remove_signatory_success":true, "eth_address": eth_address}))
}
*/

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
                .cloned()
                .map(|debug_signatory| debug_signatory.to_bytes())
                .collect::<Result<Vec<Bytes>>>()?,
        )?)
    }

    pub fn to_jsons(&self) -> Vec<DebugSignatoryJson> {
        self.iter().cloned().map(|signatory| signatory.to_json()).collect()
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
            mutable_self.push(signatory.clone());
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

    fn get(&self, eth_address: &EthAddress) -> Result<DebugSignatory> {
        let signatories = self
            .iter()
            .filter(|signatory| signatory.eth_address == *eth_address)
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

    fn replace(&self, signatory: &DebugSignatory) -> Result<Self> {
        let eth_address = signatory.eth_address;
        if self.get(&eth_address).is_ok() {
            Ok(self.remove(&eth_address).add(signatory))
        } else {
            Err(format!("Cannot replace entry, none exists with eth address: '{}'!", eth_address).into())
        }
    }

    fn increment_nonce_in_signatory_in_db<D: DatabaseInterface>(db: &D, eth_address: &EthAddress) -> Result<()> {
        let signatories = Self::get_from_db(db)?;
        signatories
            .get(eth_address)
            .map(|signatory| signatory.increment_nonce())
            .and_then(|signatory| signatories.replace(&signatory))
            .and_then(|signatories| signatories.put_in_db(db))
    }

    pub fn add_and_update_in_db<D: DatabaseInterface>(db: &D, signatory: &DebugSignatory) -> Result<()> {
        Self::get_from_db(db)
            .map(|signatories| signatories.add(signatory))
            .and_then(|signatories| signatories.put_in_db(db))
    }

    pub fn remove_and_update_in_db<D: DatabaseInterface>(db: &D, eth_address: &EthAddress) -> Result<()> {
        Self::get_from_db(db)
            .map(|signatories| signatories.remove(eth_address))
            .and_then(|signatories| signatories.put_in_db(db))
    }

    fn maybe_validate_signature_for_eth_address_and_increment_nonce_in_db<D: DatabaseInterface>(
        db: &D,
        eth_address: &EthAddress,
        bytes: &[Byte],
        signature_str: &str,
    ) -> Result<()> {
        let signatories = Self::get_from_db(db)?;
        signatories
            .get(eth_address)
            .and_then(|signatory| signatory.validate_signature(bytes, signature_str))
            .and_then(|_| Self::increment_nonce_in_signatory_in_db(db, eth_address))
    }

    pub fn maybe_validate_signature_and_increment_nonce_in_db<D: DatabaseInterface>(
        db: &D,
        bytes: &[Byte],
        signature_str: &str,
    ) -> Result<()> {
        Self::get_from_db(db)
            .map(|signatories| {
                signatories
                    .iter()
                    .map(|signatory| signatory.eth_address)
                    .collect::<Vec<_>>()
            })
            .and_then(|eth_addresses| {
                if eth_addresses
                    .iter()
                    .filter_map(|eth_address| {
                        match Self::maybe_validate_signature_for_eth_address_and_increment_nonce_in_db(
                            db,
                            eth_address,
                            bytes,
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

impl Display for DebugSignatories {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self.to_jsons()).unwrap())
    }
}

/*
#[cfg(test)]
mod tests {
    use rand::prelude::*;

    use super::*;
    use crate::{errors::AppError, test_utils::get_test_database};

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

    #[test]
    fn valid_signature_should_pass_validation() {
        use simple_logger;
        simple_logger::init().unwrap();

        let address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let signatory = DebugSignatory::new(&address);
        let bytes: Bytes = vec![0xc0, 0xff, 0xee];
        // NOTE: As gotten from etherscan signing fxnality, signing over the message
        // "b894b0bd19b5dece4bcad4b2201aee87d14f496657f859f380542535b237cfdd".
        let signature = "0x4d79044db655355772829c178cd472a2b2e9543042ab967ffd6a6e3d87d4f27a3b7a60c80a3d39ea24e9992cadaed5d9c4d61291cb566e81f1c55a531f525fd71b";
        let result = signatory.validate_signature(&bytes, signature);
        assert!(result.is_ok());
    }

    #[test]
    fn invalid_signature_should_not_pass_validation() {
        let address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let signatory = DebugSignatory::new(&address);
        let nonce = signatory.nonce;
        // NOTE: As gotten from etherscan signing fxnality, signing over the message "1".
        let signature = "0x1b18a47e64f19543b9e5b8d06f3de5e63ef0a4d99542e4cdbdb3431f38bfcf1f6ae023d4b779ada0b27f902c757ea86d75c7f59c653e69f3bf059c89670c48861b";
        let expected_error = format!(
            "Signature is not valid for eth address '{}' over nonce {}!",
            address, nonce,
        );
        let bytes: Bytes = vec![];
        match signatory.validate_signature(&bytes, signature) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_get_debug_signatory_from_signatories() {
        let signatories = get_n_random_debug_signatories(5);
        let expected_result = signatories[3].clone();
        let eth_address = expected_result.eth_address.clone();
        let result = signatories.get(&eth_address).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_fail_to_get_non_extant_debug_signatory_from_signatories() {
        let signatories = get_n_random_debug_signatories(5);
        let eth_address = EthAddress::random();
        let expected_error = format!("Could not find debug signatory with address: '{}'!", eth_address);
        match signatories.get(&eth_address) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_error_if_entry_in_signatories_twice() {
        let signatory = get_random_debug_signatory();
        // NOTE: This is the only way we can create one with a duplicate in it.
        let signatories = DebugSignatories(vec![signatory.clone(), signatory]);
        let eth_address = signatory.eth_address.clone();
        let expected_error = format!("> 1 entry found with address: '{}'!", eth_address);
        match signatories.get(&eth_address) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_fail_to_replace_non_existent_entry() {
        let signatories = get_n_random_debug_signatories(5);
        let signatory = get_random_debug_signatory();
        let expected_error = format!(
            "Cannot replace entry, none exists with eth address: '{}'!",
            signatory.eth_address
        );
        match signatories.replace(&signatory) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn should_replace_entry() {
        let signatories = get_n_random_debug_signatories(5);
        let index = 2;
        let mut signatory = signatories[index].clone();
        let eth_address = signatory.eth_address;
        signatory.nonce = signatory.nonce + 1;
        assert_ne!(signatory, signatories[index]);
        let updated_signatories = signatories.replace(&signatory).unwrap();
        let result = updated_signatories.get(&eth_address).unwrap();
        assert_eq!(result, signatory);
    }

    #[test]
    fn should_increment_nonce() {
        let signatory = get_random_debug_signatory();
        let nonce_before = signatory.nonce;
        let result = signatory.increment_nonce().nonce;
        let expected_result = nonce_before + 1;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_increment_nonce_in_entry_in_db() {
        let signatories = get_n_random_debug_signatories(5);
        let db = get_test_database();
        let index = 2;
        let signatory = signatories[index].clone();
        let eth_address = signatory.eth_address;
        let nonce_before = signatory.nonce;
        signatories.put_in_db(&db).unwrap();
        DebugSignatories::increment_nonce_in_signatory_in_db(&db, &eth_address).unwrap();
        let updated_signatories = DebugSignatories::get_from_db(&db).unwrap();
        let expected_result = nonce_before + 1;
        let result = updated_signatories.get(&eth_address).unwrap().nonce;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_maybe_validate_signature_and_increment_nonce_in_db() {
        let db = get_test_database();
        let eth_address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let bytes: Bytes = vec![];
        let signatory_1 = DebugSignatory::new(&eth_address);
        let signatory_2 = get_random_debug_signatory();
        let signatory_3 = get_random_debug_signatory();

        // NOTE: Assert the signatory we care about's nonce is 0.
        assert_eq!(signatory_1.nonce, 0);
        let signatories = DebugSignatories(vec![signatory_2.clone(), signatory_1.clone(), signatory_3.clone()]);
        signatories.put_in_db(&db).unwrap();
        // NOTE: As gotten from etherscan signing fxnality, signing over the message "0".
        let signature = "0xbc2554423224c202eebc312c8ae0c42c503ca9c0a70f3dee8b24ce79c3c3ee682d2d93c0e61e84b3a8ca93dfe8c4f89d62f0fc275c72976e420de21097ef3ebb1c";
        // NOTE: Signature should be valid, and the nonce for this signatory should be incremented.
        DebugSignatories::maybe_validate_signature_and_increment_nonce_in_db(&db, &bytes, &signature).unwrap();

        // NOTE: So lets assert that this signatory's nonce did indeed get updated.
        let updated_signatories = DebugSignatories::get_from_db(&db).unwrap();
        assert_eq!(updated_signatories.get(&eth_address).unwrap().nonce, 1);

        // NOTE: And finally, assert that the other two signatory's nonces did NOT get
        // incremented.
        assert_eq!(
            updated_signatories.get(&signatory_2.eth_address).unwrap().nonce,
            signatory_2.nonce
        );
        assert_eq!(
            updated_signatories.get(&signatory_3.eth_address).unwrap().nonce,
            signatory_3.nonce
        );
    }

    #[test]
    fn should_fail_to_maybe_validate_invalid_signature_and_thus_not_increment_nonce_in_db() {
        let db = get_test_database();
        let signatory_1 = get_random_debug_signatory();
        let signatory_2 = get_random_debug_signatory();
        let signatory_3 = get_random_debug_signatory();
        let signatories = DebugSignatories(vec![signatory_1.clone(), signatory_2.clone(), signatory_3.clone()]);
        let bytes: Bytes = vec![];
        signatories.put_in_db(&db).unwrap();

        // NOTE: This signature is by a signatory that is NOT one of the above 3.
        let signature = "0xbc2554423224c202eebc312c8ae0c42c503ca9c0a70f3dee8b24ce79c3c3ee682d2d93c0e61e84b3a8ca93dfe8c4f89d62f0fc275c72976e420de21097ef3ebb1c";

        // NOTE: And so it should error...
        let expected_error = "Signature not valid for any debug signatories!";
        match DebugSignatories::maybe_validate_signature_and_increment_nonce_in_db(&db, &bytes, &signature) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }

        // NOTE: So let's assert that this signatory's nonce did indeed get updated.
        let updated_signatories = DebugSignatories::get_from_db(&db).unwrap();

        // NOTE: Assert that none of the signatory's had their nonces incremented...
        assert_eq!(
            updated_signatories.get(&signatory_1.eth_address).unwrap().nonce,
            signatory_1.nonce
        );
        assert_eq!(
            updated_signatories.get(&signatory_2.eth_address).unwrap().nonce,
            signatory_2.nonce
        );
        assert_eq!(
            updated_signatories.get(&signatory_3.eth_address).unwrap().nonce,
            signatory_3.nonce
        );
    }
}
*/
