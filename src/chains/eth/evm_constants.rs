use serde_json::{json, Value as JsonValue};

use crate::utils::get_prefixed_db_key;

pub fn get_evm_constants_db_keys() -> JsonValue {
    json!({
        "EVM_ADDRESS_KEY":
            hex::encode(EVM_ADDRESS_KEY.to_vec()),
        "EVM_CHAIN_ID_KEY":
            hex::encode(EVM_CHAIN_ID_KEY.to_vec()),
        "EVM_GAS_PRICE_KEY":
            hex::encode(EVM_GAS_PRICE_KEY.to_vec()),
        "EVM_LINKER_HASH_KEY":
            hex::encode(EVM_LINKER_HASH_KEY.to_vec()),
        "EVM_ACCOUNT_NONCE_KEY":
            hex::encode(EVM_ACCOUNT_NONCE_KEY.to_vec()),
        "EVM_PRIVATE_KEY_DB_KEY":
            hex::encode(EVM_PRIVATE_KEY_DB_KEY.to_vec()),
        "EVM_TAIL_BLOCK_HASH_KEY":
            hex::encode(EVM_TAIL_BLOCK_HASH_KEY.to_vec()),
        "EVM_CANON_BLOCK_HASH_KEY":
            hex::encode(EVM_CANON_BLOCK_HASH_KEY.to_vec()),
        "EVM_ANY_SENDER_NONCE_KEY":
            hex::encode(EVM_ANY_SENDER_NONCE_KEY.to_vec()),
        "EVM_ANCHOR_BLOCK_HASH_KEY":
            hex::encode(EVM_ANCHOR_BLOCK_HASH_KEY.to_vec()),
        "EVM_LATEST_BLOCK_HASH_KEY":
            hex::encode(EVM_LATEST_BLOCK_HASH_KEY.to_vec()),
        "EVM_PTOKEN_GENESIS_HASH_KEY":
            hex::encode(EVM_PTOKEN_GENESIS_HASH_KEY.to_vec()),
        "EVM_CANON_TO_TIP_LENGTH_KEY":
            hex::encode(EVM_CANON_TO_TIP_LENGTH_KEY.to_vec()),
        "EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY":
            hex::encode(EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY.to_vec()),
        "EVM_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY":
            hex::encode(EVM_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY.to_vec()),
        "EVM_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY":
            hex::encode(EVM_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY.to_vec()),
        "EVM_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY":
            hex::encode(EVM_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY.to_vec()),
        "EVM_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY":
            hex::encode(EVM_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY.to_vec()),
        "EVM_ERC20_ON_EVM_ROUTER_SMART_CONTRACT_ADDRESS_KEY":
            hex::encode(EVM_ERC20_ON_EVM_ROUTER_SMART_CONTRACT_ADDRESS_KEY.to_vec()),
    })
}

lazy_static! {
    pub static ref EVM_CHAIN_ID_KEY: [u8; 32] = get_prefixed_db_key("evm-chain-id");
    pub static ref EVM_GAS_PRICE_KEY: [u8; 32] = get_prefixed_db_key("evm-gas-price");
    pub static ref EVM_ADDRESS_KEY: [u8; 32] = get_prefixed_db_key("evm-address-key");
    pub static ref EVM_LINKER_HASH_KEY: [u8; 32] = get_prefixed_db_key("evm-linker-hash-key");
    pub static ref EVM_ACCOUNT_NONCE_KEY: [u8; 32] = get_prefixed_db_key("evm-account-nonce");
    pub static ref EVM_PRIVATE_KEY_DB_KEY: [u8; 32] = get_prefixed_db_key("evm-private-key-key");
    pub static ref EVM_ANY_SENDER_NONCE_KEY: [u8; 32] = get_prefixed_db_key("evm-any-sender-nonce");
    pub static ref EVM_TAIL_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("evm-tail-block-hash-key");
    pub static ref EVM_PTOKEN_GENESIS_HASH_KEY: [u8; 32] = get_prefixed_db_key("evm-provable-ptoken");
    pub static ref EVM_CANON_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("evm-canon-block-hash-key");
    pub static ref EVM_ANCHOR_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("evm-anchor-block-hash-key");
    pub static ref EVM_LATEST_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("evm-latest-block-hash-key");
    pub static ref EVM_CANON_TO_TIP_LENGTH_KEY: [u8; 32] = get_prefixed_db_key("evm-canon-to-tip-length-key");
    pub static ref EVM_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY: [u8; 32] = get_prefixed_db_key("evm-smart-contract");
    pub static ref EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY: [u8; 32] =
        get_prefixed_db_key("evm-erc-777-proxy-contract-address-key");
    pub static ref EVM_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY: [u8; 32] =
        get_prefixed_db_key("evm-erc20-on-eos-smart-contract-address-key");
    pub static ref EVM_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY: [u8; 32] =
        get_prefixed_db_key("evm-eos-on-eth-smart-contract-address-key");
    pub static ref EVM_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY: [u8; 32] =
        get_prefixed_db_key("evm-erc20-on-evm-smart-contract-address-key");
    pub static ref EVM_ERC20_ON_EVM_ROUTER_SMART_CONTRACT_ADDRESS_KEY: [u8; 32] =
        get_prefixed_db_key("evm-erc20-on-evm-router-smart-contract-address-key");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_evm_constants_db_keys() {
        let expected_result = json!({
            "EVM_ACCOUNT_NONCE_KEY":
                "ca7f0ab19900680d76625f41854791660729bfcaf7fede763d96d4c05916ec4c",
            "EVM_ADDRESS_KEY":
                "a1e0ede222d5df7500e8580bdf0f552b55e4f95a5a1585b059adbd1fab061d73",
            "EVM_ANCHOR_BLOCK_HASH_KEY":
                "0a28ac19c3f6ed77642240975ff3d553290e62785b9070e81fad38012d346bae",
            "EVM_ANY_SENDER_NONCE_KEY":
                "960d6c59b7c81545d0fcedd4a4e84102b306bef422b6f06b38c452df19b0673f",
            "EVM_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY":
                "1a2270b3479ad2a676751ecbf17c8468ab64854d265d1ba8107e042e70a5c422",
            "EVM_CANON_BLOCK_HASH_KEY":
                "bc262de20ac1da20589be1d2464e9658bf9d5ab193ad65ff5be69008bbbc8ee2",
            "EVM_CANON_TO_TIP_LENGTH_KEY":
                "2ee78935508a7ae8327e1ec867d23813042f70e78ac5dafa05d00ed3a81eb7d7",
            "EVM_CHAIN_ID_KEY":
                "b302d7601e077a277f2d1e100c959ba2d63989531b47468bbeef4c9faa57d3c9",
            "EVM_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY":
                "3afdaa0cf2f37afa64f93623c3b25778c9cde2f6a71af4818c78ab54c4731144",
            "EVM_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY":
                "e06e403795bcba77bcaa7ae8e22a7149e69c7fe8eb7db5e81e4c80a268594fdb",
            "EVM_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY":
                "a7e4cd0d8bf1e96eaff6b8f74cb8786c834330f34cf209597ca988f5d724b4a7",
            "EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY":
                "0e5e8342356bb9f5b6f6b1a681c544c12838053a450bb97bed1d3a7a8e9a86ec",
            "EVM_GAS_PRICE_KEY":
                "b4dbeaf50ce099e52bd74571377dc97df7f25db7b981babcea4c0292035f58ba",
            "EVM_LATEST_BLOCK_HASH_KEY":
                "9a4dd10e7fc05b39c5c66698d808005e9bc678bf3d7816741b25ddddf93092a7",
            "EVM_LINKER_HASH_KEY":
                "b4ed69606ec2498bc6f8ea41a8ec6f46181d36617966c5083345115e0b7b964c",
            "EVM_PRIVATE_KEY_DB_KEY":
                "fa8338b621f949093c2880563aa678a8407ce0c78c1d75b9fec11768b042eba7",
            "EVM_PTOKEN_GENESIS_HASH_KEY":
                "2571ca7ce4ca58cbd74f2ec4d971bc90925a9c2305481798bab1a8a7e7ad67bc",
            "EVM_TAIL_BLOCK_HASH_KEY":
                "0bfa597048f0580d7782b60c89e596410b708ed843c5391f53fbfd6e947bccb4",
            "EVM_ERC20_ON_EVM_ROUTER_SMART_CONTRACT_ADDRESS_KEY":
                "a302e0107ddf40eea3e0598779fab70581181064d07c64af35d29202705905f2",
        });
        let result = get_evm_constants_db_keys();
        assert_eq!(result, expected_result);
    }
}
