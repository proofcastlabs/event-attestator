use ethereum_types::{Address as EthAddress, H256};

use crate::{
    chains::eth::eth_crypto::eth_signature::EthSignature,
    core_type::CoreType,
    crypto_utils::keccak_hash_bytes,
    debug_functions::debug_signers::debug_signatory::DebugSignatory,
    types::{Bytes, Result},
    utils::strip_hex_prefix,
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

    fn hash_with_eth_prefix(
        &self,
        core_type: &CoreType,
        debug_command_hash: &H256,
        use_hex_prefix: bool,
    ) -> Result<H256> {
        self.hash_to_hex(core_type, debug_command_hash)
            .map(|message| {
                let stripped_message = strip_hex_prefix(&message);
                if use_hex_prefix {
                    format!("0x{}", stripped_message)
                } else {
                    stripped_message
                }
            })
            .map(|message| Self::get_eth_prefixed_message_bytes(&message))
            .map(|bytes| H256::from_slice(&bytes))
    }

    fn recover_signer_address(
        &self,
        signature: &EthSignature,
        core_type: &CoreType,
        debug_command_hash: &H256,
    ) -> Result<EthAddress> {
        info!("✔ Recovering signature with NO ETH prefix...");
        let address = signature.recover_signer_address(&self.hash(core_type, debug_command_hash)?)?;
        debug!("Recovered address: {}", address);
        Ok(address)
    }

    fn recover_signer_address_using_eth_prefix_and_hex_prefix(
        &self,
        signature: &EthSignature,
        core_type: &CoreType,
        debug_command_hash: &H256,
    ) -> Result<EthAddress> {
        info!("✔ Recovering signature with ETH prefix AND hex prefix...");
        let address =
            signature.recover_signer_address(&self.hash_with_eth_prefix(core_type, debug_command_hash, true)?)?;
        debug!("Recovered address: {}", address);
        Ok(address)
    }

    fn recover_signer_address_using_eth_prefix_and_no_hex_prefix(
        &self,
        signature: &EthSignature,
        core_type: &CoreType,
        debug_command_hash: &H256,
    ) -> Result<EthAddress> {
        info!("✔ Recovering signature with ETH prefix and NO hex prefix...");
        let address =
            signature.recover_signer_address(&self.hash_with_eth_prefix(core_type, debug_command_hash, false)?)?;
        debug!("Recovered address: {}", address);
        Ok(address)
    }

    pub fn validate(&self, signature: &EthSignature, core_type: &CoreType, debug_command_hash: &H256) -> Result<()> {
        let recovered_addresses = vec![
            self.recover_signer_address(signature, core_type, debug_command_hash)?,
            self.recover_signer_address_using_eth_prefix_and_hex_prefix(signature, core_type, debug_command_hash)?,
            self.recover_signer_address_using_eth_prefix_and_no_hex_prefix(signature, core_type, debug_command_hash)?,
        ];
        if recovered_addresses.contains(&self.eth_address) {
            Ok(())
        } else {
            Err("Could not validate debug signature!".into())
        }
    }
}

#[cfg(test)]
use crate::chains::eth::{eth_crypto::eth_private_key::EthPrivateKey, eth_traits::EthSigningCapabilities};

#[cfg(test)]
impl DebugSignatory {
    pub fn sign_with_eth_prefix(
        &self,
        pk: &EthPrivateKey,
        core_type: &CoreType,
        debug_command_hash: &H256,
    ) -> Result<EthSignature> {
        self.hash_with_eth_prefix(core_type, debug_command_hash, false)
            .and_then(|hash| pk.sign_hash_and_set_eth_recovery_param(hash))
    }

    #[cfg(test)]
    pub fn sign(&self, pk: &EthPrivateKey, core_type: &CoreType, debug_command_hash: &H256) -> Result<EthSignature> {
        self.hash(core_type, debug_command_hash)
            .and_then(|hash| pk.sign_hash_and_set_eth_recovery_param(hash))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::{
        debug_functions::debug_signers::test_utils::{
            get_sample_debug_command_hash,
            get_sample_debug_signatory,
            get_sample_private_key,
        },
        errors::AppError,
    };

    #[test]
    fn should_sign_debug_signatory_hash() {
        let pk = get_sample_private_key();
        let core_type = CoreType::BtcOnInt;
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let result = hex::encode(&signatory.sign(&pk, &core_type, &debug_command_hash).unwrap().to_vec());
        let expected_result = "7ee719e38908f63a26aca7a3957b7573156e01e9a853ada12ae877789d4c95a06334c13f397b050a5318867325cf60ebdcce1145c9f786ca522cb3d9bc9ab5a91b";
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_recover_correct_address_from_signature_over_debug_signatory_hash() {
        let pk = get_sample_private_key();
        let core_type = CoreType::BtcOnInt;
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = signatory.sign(&pk, &core_type, &debug_command_hash).unwrap();
        let result = signatory
            .recover_signer_address(&signature, &core_type, &debug_command_hash)
            .unwrap();
        let expected_result = pk.to_public_key().to_address();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_validate_debug_signature() {
        let pk = get_sample_private_key();
        let core_type = CoreType::BtcOnInt;
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = signatory.sign(&pk, &core_type, &debug_command_hash).unwrap();
        let result = signatory.validate(&signature, &core_type, &debug_command_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn should_sign_with_eth_prefix() {
        let core_type = CoreType::BtcOnInt;
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let pk = get_sample_private_key();
        let result = debug_signatory
            .sign_with_eth_prefix(&pk, &core_type, &debug_command_hash)
            .unwrap();
        let expected_result = EthSignature::from_str("9af3fde7df5987f7d43b4b27948d089eac4495595e731891fb3b9e7a78eca0a151fc6bfa50f75783e65197c4a1ff07969112df0b6b1d3e72538c566aed4dab221b").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_recover_address_from_eth_prefixed_signature() {
        let pk = get_sample_private_key();
        let core_type = CoreType::BtcOnInt;
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = signatory
            .sign_with_eth_prefix(&pk, &core_type, &debug_command_hash)
            .unwrap();
        let result = signatory
            .recover_signer_address_using_eth_prefix_and_no_hex_prefix(&signature, &core_type, &debug_command_hash)
            .unwrap();
        let expected_result = pk.to_public_key().to_address();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_validate_with_eth_prefix_signature() {
        let core_type = CoreType::BtcOnInt;
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let pk = get_sample_private_key();
        let eth_prefixed_signature = debug_signatory
            .sign_with_eth_prefix(&pk, &core_type, &debug_command_hash)
            .unwrap();
        let result = debug_signatory.validate(&eth_prefixed_signature, &core_type, &debug_command_hash);
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_to_validate_signature_using_wrong_eth_address() {
        let pk = get_sample_private_key();
        let core_type = CoreType::BtcOnInt;
        let debug_signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = debug_signatory.sign(&pk, &core_type, &debug_command_hash).unwrap();
        let wrong_debug_signatory = DebugSignatory::random();
        assert_ne!(wrong_debug_signatory.eth_address, pk.to_public_key().to_address());
        let expected_error = "Could not validate debug signature!".to_string();
        match wrong_debug_signatory.validate(&signature, &core_type, &debug_command_hash) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
