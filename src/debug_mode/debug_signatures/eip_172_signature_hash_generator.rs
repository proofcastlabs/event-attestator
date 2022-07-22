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
    pub fn to_eip_712_typed_data(&self, debug_command_hash: &H256) -> Result<EIP712> {
        let s = format!(
            r#"{{
            "primaryType": "DebugSignatory",
            "domain": {{
                "name": "PTokens Debug Signatory",
                "version": "1",
                "chainId": "0x1",
                "verifyingContract": "{}"
            }},
            "message": {{
                "signerNonce": "{}",
                "signerName": "{}",
                "signerAddress": "{}",
                "debugCmdHash": "{}"
            }},
            "types": {{
                "EIP712Domain": [
                    {{ "name": "name", "type": "string" }},
                    {{ "name": "version", "type": "string" }},
                    {{ "name": "chainId", "type": "uint256" }},
                    {{ "name": "verifyingContract", "type": "address" }}
                ],
                "DebugSignatory": [
                    {{ "name": "signerNonce", "type": "uint256" }},
                    {{ "name": "signerName", "type": "string" }},
                    {{ "name": "signerAddress", "type": "address" }},
                    {{ "name": "debugCmdHash", "type": "bytes32" }}
                ]
            }}
        }}"#,
            // NOTE: This is a required field, but we neither have no need one.
            convert_eth_address_to_string(&EthAddress::zero()),
            format_args!("0x{}", hex::encode(self.get_nonce_as_bytes())),
            self.name,
            convert_eth_address_to_string(&self.eth_address),
            format_args!("0x{}", hex::encode(debug_command_hash)),
        );
        Ok(serde_json::from_str(&s)?)
    }

    pub fn hash(&self, debug_command_hash: &H256) -> Result<H256> {
        self.to_eip_712_typed_data(debug_command_hash)
            .and_then(|eip_712_typed_data| Ok(hash_structured_data(eip_712_typed_data)?))
            .map(|bytes| H256::from_slice(&bytes))
    }

    // NOTE: The `debug_command_hash` is the hash of the `cli_args` struct parsed by docopt in the
    // app which consumes this core library.
    pub fn hash_to_hex(&self, debug_command_hash: &H256) -> Result<String> {
        self.hash(debug_command_hash).map(hex::encode)
    }
}

#[cfg(test)]
mod tests {
    use serde_json;
    use web3::signing::recover as recover_signer_address;

    use super::*;
    use crate::{
        chains::eth::eth_utils::convert_hex_to_eth_address,
        debug_mode::debug_signatures::test_utils::{get_sample_debug_command_hash, get_sample_debug_signatory},
    };

    #[test]
    fn should_get_debug_signatory_hash() {
        let debug_command_hash = get_sample_debug_command_hash();
        let signatory = get_sample_debug_signatory();
        let result = signatory.hash_to_hex(&debug_command_hash).unwrap();
        let expected_result = "5bfbc8061ca361003107560a5bbc4351886829eac84826b935d6342ee6db6967";
        assert_eq!(result, expected_result);
    }
}
