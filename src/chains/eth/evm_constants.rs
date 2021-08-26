use serde_json::{json, Value as JsonValue};

use crate::utils::get_prefixed_db_key;

pub fn get_evm_constants_db_keys() -> JsonValue {
    json!({
        "EVM_ADDRESS_KEY": hex::encode(EVM_ADDRESS_KEY.to_vec()),
        "EVM_CHAIN_ID_KEY": hex::encode(EVM_CHAIN_ID_KEY.to_vec()),
        "EVM_GAS_PRICE_KEY": hex::encode(EVM_GAS_PRICE_KEY.to_vec()),
        "EVM_LINKER_HASH_KEY": hex::encode(EVM_LINKER_HASH_KEY.to_vec()),
        "EVM_ACCOUNT_NONCE_KEY": hex::encode(EVM_ACCOUNT_NONCE_KEY.to_vec()),
        "EVM_PRIVATE_KEY_DB_KEY": hex::encode(EVM_PRIVATE_KEY_DB_KEY.to_vec()),
        "EVM_TAIL_BLOCK_HASH_KEY": hex::encode(EVM_TAIL_BLOCK_HASH_KEY.to_vec()),
        "EVM_CANON_BLOCK_HASH_KEY": hex::encode(EVM_CANON_BLOCK_HASH_KEY.to_vec()),
        "EVM_ANY_SENDER_NONCE_KEY": hex::encode(EVM_ANY_SENDER_NONCE_KEY.to_vec()),
        "EVM_ANCHOR_BLOCK_HASH_KEY": hex::encode(EVM_ANCHOR_BLOCK_HASH_KEY.to_vec()),
        "EVM_LATEST_BLOCK_HASH_KEY": hex::encode(EVM_LATEST_BLOCK_HASH_KEY.to_vec()),
        "EVM_PTOKEN_GENESIS_HASH_KEY": hex::encode(EVM_PTOKEN_GENESIS_HASH_KEY.to_vec()),
        "EVM_CANON_TO_TIP_LENGTH_KEY": hex::encode(EVM_CANON_TO_TIP_LENGTH_KEY.to_vec()),
        "EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY": hex::encode(EVM_ERC777_PROXY_CONTACT_ADDRESS_KEY.to_vec()),
        "EVM_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY": hex::encode(EVM_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY.to_vec()),
        "EVM_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY": hex::encode(EVM_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY.to_vec()),
        "EVM_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY": hex::encode(EVM_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY.to_vec()),
    })
}

lazy_static! {
    pub static ref EVM_CHAIN_ID_KEY: [u8; 32] = get_prefixed_db_key("evm-chain-id");
    pub static ref EVM_GAS_PRICE_KEY: [u8; 32] = get_prefixed_db_key("evm-gas-price");
    pub static ref EVM_ADDRESS_KEY: [u8; 32] = get_prefixed_db_key("evm-address-key");
    pub static ref EVM_LINKER_HASH_KEY: [u8; 32] = get_prefixed_db_key("evm-linker-hash-key");
    pub static ref EVM_ANY_SENDER_NONCE_KEY: [u8; 32] = get_prefixed_db_key("evm-any-sender-nonce");
    pub static ref EVM_ACCOUNT_NONCE_KEY: [u8; 32] = get_prefixed_db_key("evm-account-nonce");
    pub static ref EVM_PTOKEN_GENESIS_HASH_KEY: [u8; 32] = get_prefixed_db_key("evm-provable-ptoken");
    pub static ref EVM_PRIVATE_KEY_DB_KEY: [u8; 32] = get_prefixed_db_key("evm-private-key-key");
    pub static ref EVM_CANON_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("evm-canon-block-hash-key");
    pub static ref EVM_TAIL_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("evm-tail-block-hash-key");
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
}
