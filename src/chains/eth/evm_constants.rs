create_db_keys_and_json!(
    "Evm";
    "EVM_CHAIN_ID_KEY" => "evm-chain-id",
    "EVM_GAS_PRICE_KEY" => "evm-gas-price",
    "EVM_ADDRESS_KEY" => "evm-address-key",
    "EVM_LINKER_HASH_KEY" => "evm-linker-hash-key",
    "EVM_ACCOUNT_NONCE_KEY" => "evm-account-nonce",
    "EVM_PRIVATE_KEY_DB_KEY" => "evm-private-key-key",
    "EVM_ANY_SENDER_NONCE_KEY" => "evm-any-sender-nonce",
    "EVM_TAIL_BLOCK_HASH_KEY" => "evm-tail-block-hash-key",
    "EVM_PTOKEN_GENESIS_HASH_KEY" => "evm-provable-ptoken",
    "EVM_CANON_BLOCK_HASH_KEY" => "evm-canon-block-hash-key",
    "EVM_ANCHOR_BLOCK_HASH_KEY" => "evm-anchor-block-hash-key",
    "EVM_LATEST_BLOCK_HASH_KEY" => "evm-latest-block-hash-key",
    "EVM_CANON_TO_TIP_LENGTH_KEY" => "evm-canon-to-tip-length-key",
    "EVM_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY" => "evm-smart-contract",
    "EVM_ROUTER_SMART_CONTRACT_ADDRESS_KEY" => "eth-router-smart-contract-address-key",
    "EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY" => "evm-erc-777-proxy-contract-address-key",
    "EVM_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY" => "evm-eos-on-eth-smart-contract-address-key",
    "EVM_INT_ON_EVM_SMART_CONTRACT_ADDRESS_KEY" => "eth-int-on-evm-smart-contract-address-key",
    "EVM_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY" => "evm-erc20-on-eos-smart-contract-address-key",
    "EVM_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY" => "evm-erc20-on-evm-smart-contract-address-key"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evm_db_keys_should_stay_consistent() {
        #[rustfmt::skip]
        let expected_result = EvmDatabaseKeysJson {
            EVM_ACCOUNT_NONCE_KEY:
               "ca7f0ab19900680d76625f41854791660729bfcaf7fede763d96d4c05916ec4c".to_string(),
            EVM_ADDRESS_KEY:
               "a1e0ede222d5df7500e8580bdf0f552b55e4f95a5a1585b059adbd1fab061d73".to_string(),
            EVM_ANCHOR_BLOCK_HASH_KEY:
               "0a28ac19c3f6ed77642240975ff3d553290e62785b9070e81fad38012d346bae".to_string(),
            EVM_ANY_SENDER_NONCE_KEY:
               "960d6c59b7c81545d0fcedd4a4e84102b306bef422b6f06b38c452df19b0673f".to_string(),
            EVM_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY:
               "1a2270b3479ad2a676751ecbf17c8468ab64854d265d1ba8107e042e70a5c422".to_string(),
            EVM_CANON_BLOCK_HASH_KEY:
               "bc262de20ac1da20589be1d2464e9658bf9d5ab193ad65ff5be69008bbbc8ee2".to_string(),
            EVM_CANON_TO_TIP_LENGTH_KEY:
               "2ee78935508a7ae8327e1ec867d23813042f70e78ac5dafa05d00ed3a81eb7d7".to_string(),
            EVM_CHAIN_ID_KEY:
               "b302d7601e077a277f2d1e100c959ba2d63989531b47468bbeef4c9faa57d3c9".to_string(),
            EVM_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY:
               "3afdaa0cf2f37afa64f93623c3b25778c9cde2f6a71af4818c78ab54c4731144".to_string(),
            EVM_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY:
               "e06e403795bcba77bcaa7ae8e22a7149e69c7fe8eb7db5e81e4c80a268594fdb".to_string(),
            EVM_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY:
               "a7e4cd0d8bf1e96eaff6b8f74cb8786c834330f34cf209597ca988f5d724b4a7".to_string(),
            EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY:
               "0e5e8342356bb9f5b6f6b1a681c544c12838053a450bb97bed1d3a7a8e9a86ec".to_string(),
            EVM_GAS_PRICE_KEY:
               "b4dbeaf50ce099e52bd74571377dc97df7f25db7b981babcea4c0292035f58ba".to_string(),
            EVM_INT_ON_EVM_SMART_CONTRACT_ADDRESS_KEY:
               "a1552e7ee400c2adf873879fc3efefea72db11307ad3c873506e1f3be8fd31db".to_string(),
            EVM_LATEST_BLOCK_HASH_KEY:
               "9a4dd10e7fc05b39c5c66698d808005e9bc678bf3d7816741b25ddddf93092a7".to_string(),
            EVM_LINKER_HASH_KEY:
               "b4ed69606ec2498bc6f8ea41a8ec6f46181d36617966c5083345115e0b7b964c".to_string(),
            EVM_PRIVATE_KEY_DB_KEY:
               "fa8338b621f949093c2880563aa678a8407ce0c78c1d75b9fec11768b042eba7".to_string(),
            EVM_PTOKEN_GENESIS_HASH_KEY:
               "2571ca7ce4ca58cbd74f2ec4d971bc90925a9c2305481798bab1a8a7e7ad67bc".to_string(),
            EVM_TAIL_BLOCK_HASH_KEY:
               "0bfa597048f0580d7782b60c89e596410b708ed843c5391f53fbfd6e947bccb4".to_string(),
            EVM_ROUTER_SMART_CONTRACT_ADDRESS_KEY:
                "7e4ba9ad69fafede39d72a5e5d05953c4261d16ede043978031bc425d2e3b1d2".to_string(),
        };
        let result = EvmDatabaseKeysJson::new();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn eth_router_smart_contract_addres_key_should_match_evm_router_smart_contract_address_key() {
        use crate::chains::eth::{
            eth_constants::ETH_ROUTER_SMART_CONTRACT_ADDRESS_KEY,
            evm_constants::EVM_ROUTER_SMART_CONTRACT_ADDRESS_KEY,
        };
        assert_eq!(
            *ETH_ROUTER_SMART_CONTRACT_ADDRESS_KEY,
            *EVM_ROUTER_SMART_CONTRACT_ADDRESS_KEY
        );
    }
}
