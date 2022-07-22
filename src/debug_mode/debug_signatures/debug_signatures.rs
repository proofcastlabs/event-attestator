use eip_712::{hash_structured_data, EIP712};
use ethereum_types::{Address as EthAddress, H256};

use crate::{
    chains::eth::{
        eth_crypto::{eth_private_key::EthPrivateKey, eth_signature::EthSignature},
        eth_traits::EthSigningCapabilities,
        eth_utils::convert_eth_address_to_string,
    },
    debug_mode::debug_signatures::debug_signatory::DebugSignatory,
    types::{Byte, Result},
};

impl DebugSignatory {
    pub fn sign(&self, pk: &EthPrivateKey, debug_command_hash: &H256) -> Result<EthSignature> {
        self.hash(debug_command_hash)
            .and_then(|hash| pk.sign_hash_and_set_eth_recovery_param(hash))
    }

    pub fn recover_signer_address(&self, signature: &EthSignature, debug_command_hash: &H256) -> Result<EthAddress> {
        signature.recover_signer_address(&self.hash(debug_command_hash)?)
    }

    pub fn validate(
        &self,
        signature: &EthSignature,
        debug_command_hash: &H256,
        eth_address: &EthAddress,
    ) -> Result<()> {
        let recovered_addresses = vec![
            self.recover_signer_address(signature, debug_command_hash)?,
            // TODO recover it WITH the prefix too!
        ];
        if recovered_addresses.contains(eth_address) {
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
            get_sample_debug_command_hash,
            get_sample_debug_signatory,
            get_sample_private_key,
        },
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
        let expected_result = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_validate_debug_signature() {
        let pk = get_sample_private_key();
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let signature = signatory.sign(&pk, &debug_command_hash).unwrap();
        let eth_address = convert_hex_to_eth_address("0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC").unwrap();
        let result = signatory.validate(&signature, &debug_command_hash, &eth_address);
        assert!(result.is_ok());
    }
}
