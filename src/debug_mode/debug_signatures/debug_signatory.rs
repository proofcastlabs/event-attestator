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

impl Display for DebugSignatory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().to_json())
    }
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

    fn from_json(json: &DebugSignatoryJson) -> Result<Self> {
        Ok(Self {
            nonce: json.nonce,
            name: json.name.clone(),
            eth_address: convert_hex_to_eth_address(&json.eth_address)?,
        })
    }

    pub fn to_json(self) -> DebugSignatoryJson {
        DebugSignatoryJson {
            nonce: self.nonce,
            name: self.name.clone(),
            eth_address: format!("0x{}", hex::encode(self.eth_address)),
        }
    }

    pub fn to_bytes(self) -> Result<Bytes> {
        self.to_json().to_bytes()
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        DebugSignatoryJson::from_bytes(bytes).and_then(|json| Self::from_json(&json))
    }

    // FIXME We could feasibly try and recover the address as if it was signed with an ETH prefix
    // too?
    fn get_eth_prefixed_message_bytes(message: &str) -> Bytes {
        let eth_prefixed_message_bytes =
            keccak_hash_bytes(format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message).as_bytes())[..]
                .to_vec();
        debug!(
            "ETH prefixed message bytes: {}",
            hex::encode(&eth_prefixed_message_bytes)
        );
        eth_prefixed_message_bytes
    }

    pub fn get_nonce_as_bytes(&self) -> Bytes {
        let nonce_as_bytes = self.nonce.to_be_bytes().to_vec();
        debug!(
            "Nonce: {}, nonce as BE bytes: {}",
            self.nonce,
            hex::encode(&nonce_as_bytes)
        );
        nonce_as_bytes
    }

    fn get_bytes_to_hash(&self, bytes: &[Byte]) -> Bytes {
        let bytes_to_hash = vec![self.get_nonce_as_bytes(), bytes.to_vec()].concat();
        debug!("Bytes to hash: {}", hex::encode(&bytes_to_hash));
        bytes_to_hash
    }

    fn get_message_to_sign_as_hex(&self, bytes: &[Byte]) -> String {
        let message_to_sign_as_hex = hex::encode(self.get_bytes_to_hash(bytes));
        debug!("Message to sign: {}", message_to_sign_as_hex);
        message_to_sign_as_hex
    }

    fn get_signature_message_bytes(&self, bytes: &[Byte]) -> Bytes {
        Self::get_eth_prefixed_message_bytes(&self.get_message_to_sign_as_hex(bytes))
    }

    fn recover_eth_addresses_from_signature(
        &self,
        bytes: &[Byte],
        signature_bytes: &[Byte],
    ) -> Result<Vec<EthAddress>> {
        // NOTE: We just calculate the address using BOTH recovery IDs, and thus we are chain
        // agnostic w/r/t to the `v` param of an EVM compliant signature.
        let signature_message_bytes = self.get_signature_message_bytes(bytes);
        let recovered_eth_addresses = vec![
            recover(&signature_message_bytes, &signature_bytes[..64], 0)?,
            recover(&signature_message_bytes, &signature_bytes[..64], 1)?,
        ];
        debug!("Recovered ETH addresses: {:?}", recovered_eth_addresses);
        Ok(recovered_eth_addresses)
    }

    fn get_signature_bytes(signature: &str) -> Result<Bytes> {
        const SIGNATURE_LENGTH: usize = 65;
        let bytes = hex::decode(strip_hex_prefix(signature))?;
        if bytes.len() != SIGNATURE_LENGTH {
            Err(format!("Signature must be {} bytes long!", SIGNATURE_LENGTH).into())
        } else {
            Ok(bytes)
        }
    }

    pub fn validate_signature(&self, bytes: &[Byte], signature: &str) -> Result<()> {
        Self::get_signature_bytes(signature)
            .and_then(|signature_bytes| self.recover_eth_addresses_from_signature(bytes, &signature_bytes))
            .map(|eth_addresses| eth_addresses.contains(&self.eth_address))
            .and_then(|is_valid| {
                if is_valid {
                    Ok(())
                } else {
                    Err(format!(
                        "Signature is not valid for eth address '{}' over nonce {}!",
                        self.eth_address, self.nonce
                    )
                    .into())
                }
            })
    }
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebugSignatoryJson {
    pub nonce: u64,
    pub name: String,
    pub eth_address: String,
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
