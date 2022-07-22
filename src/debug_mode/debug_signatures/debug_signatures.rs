use eip_712::{hash_structured_data, EIP712};
use ethereum_types::{Address as EthAddress, H256};

use crate::{
    chains::eth::{
        eth_crypto::{eth_private_key::EthPrivateKey, eth_signature::EthSignature},
        eth_traits::EthSigningCapabilities,
        eth_utils::convert_eth_address_to_string,
    },
    crypto_utils::keccak_hash_bytes,
    debug_mode::debug_signatures::debug_signatory::DebugSignatory,
    types::{Byte, Bytes, Result},
};

impl DebugSignatory {
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

    fn hash_with_eth_prefix(&self, debug_command_hash: &H256) -> Result<H256> {
        self.hash_to_hex(debug_command_hash)
            .map(|message| Self::get_eth_prefixed_message_bytes(&message))
            .map(|bytes| H256::from_slice(&bytes))
    }

    pub fn sign_with_eth_prefix(&self, pk: &EthPrivateKey, debug_command_hash: &H256) -> Result<EthSignature> {
        self.hash_with_eth_prefix(debug_command_hash)
            .and_then(|hash| pk.sign_hash_and_set_eth_recovery_param(hash))
    }

    pub fn sign(&self, pk: &EthPrivateKey, debug_command_hash: &H256) -> Result<EthSignature> {
        self.hash(debug_command_hash)
            .and_then(|hash| pk.sign_hash_and_set_eth_recovery_param(hash))
    }

    pub fn recover_signer_address(&self, signature: &EthSignature, debug_command_hash: &H256) -> Result<EthAddress> {
        signature.recover_signer_address(&self.hash(debug_command_hash)?)
    }

    pub fn recover_signer_address_using_eth_prefix(
        &self,
        signature: &EthSignature,
        debug_command_hash: &H256,
    ) -> Result<EthAddress> {
        signature.recover_signer_address(&self.hash_with_eth_prefix(debug_command_hash)?)
    }

    pub fn validate(&self, signature: &EthSignature, debug_command_hash: &H256) -> Result<()> {
        let recovered_addresses = vec![
            self.recover_signer_address(signature, debug_command_hash)?,
            self.recover_signer_address_using_eth_prefix(signature, debug_command_hash)?,
        ];
        if recovered_addresses.contains(&self.eth_address) {
            Ok(())
        } else {
            Err("Could not validate debug signature!".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json;
    use web3::signing::recover as recover_signer_address;

    use super::*;
    use crate::{
        chains::eth::eth_utils::convert_hex_to_eth_address,
        debug_mode::debug_signatures::test_utils::{
            get_random_debug_signatory,
            get_sample_debug_command_hash,
            get_sample_debug_signatory,
            get_sample_private_key,
        },
        errors::AppError,
    };

    #[test]
    fn should_sign_debug_signatory_hash() {
        let pk = get_sample_private_key();
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let result = hex::encode(&signatory.sign(&pk, &debug_command_hash).unwrap().to_vec());
        let expected_result = "8904306432eaf5948b046341c7a920b6a9432db1be58873da302006fcd4fdcd1628c7eb55a5f74d451e20b60a53e4b8318c288614d061f67cf73a460a1dde6e51b";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_recover_correct_address_from_signature_over_debug_signatory_hash() {
        let pk = get_sample_private_key();
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = signatory.sign(&pk, &debug_command_hash).unwrap();
        let result = signatory
            .recover_signer_address(&signature, &debug_command_hash)
            .unwrap();
        let expected_result = pk.to_public_key().to_address();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_validate_debug_signature() {
        let pk = get_sample_private_key();
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = signatory.sign(&pk, &debug_command_hash).unwrap();
        let result = signatory.validate(&signature, &debug_command_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn should_sign_with_eth_prefix() {
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let pk = get_sample_private_key();
        let result = debug_signatory.sign_with_eth_prefix(&pk, &debug_command_hash).unwrap();
        let expected_result = EthSignature::from_hex("0xda1a3b8f1bb8c0964b15785b5408ca3dfe35ed512d860d03bc543656e0c8f2a72c550b23a15b4c6624b3625217380ce1849e85710278ddd4aaee5d8b4f26d1521c").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_recover_address_from_eth_prefixed_signature() {
        let pk = get_sample_private_key();
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = signatory.sign_with_eth_prefix(&pk, &debug_command_hash).unwrap();
        let result = signatory
            .recover_signer_address_using_eth_prefix(&signature, &debug_command_hash)
            .unwrap();
        let expected_result = pk.to_public_key().to_address();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_validate_with_eth_prefix_signature() {
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let pk = get_sample_private_key();
        let eth_prefixed_signature = debug_signatory.sign_with_eth_prefix(&pk, &debug_command_hash).unwrap();
        let result = debug_signatory.validate(&eth_prefixed_signature, &debug_command_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_to_validate_signature_using_wrong_eth_address() {
        let pk = get_sample_private_key();
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = debug_signatory.sign(&pk, &debug_command_hash).unwrap();
        let wrong_debug_signatory = get_random_debug_signatory();
        assert_ne!(wrong_debug_signatory.eth_address, pk.to_public_key().to_address());
        let expected_error = "Could not validate debug signature!".to_string();
        match wrong_debug_signatory.validate(&signature, &debug_command_hash) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
