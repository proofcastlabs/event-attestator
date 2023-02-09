use common::{
    chains::eth::eth_utils::{convert_eth_address_to_string, convert_h256_to_string},
    core_type::CoreType,
    types::Result,
};
use eip_712::{hash_structured_data, EIP712};
use ethereum_types::{Address as EthAddress, H256};

use crate::DebugSignatory;

impl DebugSignatory {
    fn to_eip_712_typed_data(&self, core_type: &CoreType, debug_command_hash: &H256) -> Result<EIP712> {
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
                "coreType": "{}",
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
                    {{ "name": "coreType", "type": "string" }},
                    {{ "name": "signerNonce", "type": "uint256" }},
                    {{ "name": "signerName", "type": "string" }},
                    {{ "name": "signerAddress", "type": "address" }},
                    {{ "name": "debugCmdHash", "type": "bytes32" }}
                ]
            }}
        }}"#,
            // NOTE: This is a required field, but we neither have no need one.
            convert_eth_address_to_string(&EthAddress::zero()),
            core_type,
            format_args!("0x{:x}", self.nonce),
            self.name,
            convert_eth_address_to_string(&self.eth_address),
            format_args!("0x{}", hex::encode(debug_command_hash)),
        );
        Ok(serde_json::from_str(&s)?)
    }

    pub fn hash(&self, core_type: &CoreType, debug_command_hash: &H256) -> Result<H256> {
        self.to_eip_712_typed_data(core_type, debug_command_hash)
            .and_then(|eip_712_typed_data| Ok(hash_structured_data(eip_712_typed_data)?))
            .map(|bytes| H256::from_slice(&bytes))
    }

    // NOTE: The `debug_command_hash` is the hash of the `cli_args` struct parsed by docopt in the
    // app which consumes this core library.
    pub fn hash_to_hex(&self, core_type: &CoreType, debug_command_hash: &H256) -> Result<String> {
        Ok(convert_h256_to_string(&self.hash(core_type, debug_command_hash)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{get_sample_debug_command_hash, get_sample_debug_signatory};

    #[test]
    fn should_get_debug_signatory_hash() {
        let core_type = CoreType::BtcOnInt;
        let signatory = get_sample_debug_signatory();
        let debug_command_hash = get_sample_debug_command_hash();
        let result = signatory.hash_to_hex(&core_type, &debug_command_hash).unwrap();
        let expected_result = "0xe6dfc2ae5d619ba28e40c0778982d7ffb15bb081053d549372a98fc81c367b21";
        assert_eq!(result, expected_result);
    }
}
