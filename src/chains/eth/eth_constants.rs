use paste::paste;
use serde::{Deserialize, Serialize};
pub use serde_json::{json, Value as JsonValue};

use crate::utils::get_prefixed_db_key;

pub const ZERO_ETH_VALUE: usize = 0;
pub const ETH_TAIL_LENGTH: u64 = 100;
pub const VALUE_FOR_MINTING_TX: usize = 0;
pub const VALUE_FOR_PTOKEN_DEPLOY: usize = 0;
pub const ETH_WORD_SIZE_IN_BYTES: usize = 32;
pub const ARBITRUM_GAS_MULTIPLIER: usize = 10;
pub const ETH_ADDRESS_SIZE_IN_BYTES: usize = 20;
pub const MAX_BYTES_FOR_ETH_USER_DATA: usize = 2000;
pub const GAS_LIMIT_FOR_PTOKEN_DEPLOY: usize = 4_000_000;
pub const ETH_MESSAGE_PREFIX: &[u8; 26] = b"\x19Ethereum Signed Message:\n";
pub const ETH_CORE_IS_INITIALIZED_JSON: &str = "{eth_core_initialized:true}";
pub const EVM_CORE_IS_INITIALIZED_JSON: &str = "{evm_core_initialized:true}";
pub const PREFIXED_MESSAGE_HASH_LEN: &[u8; 2] = b"32";

macro_rules! create_eth_db_keys {
    ($($key:expr => $value:expr),*) => {
        paste! {
            lazy_static! {
                $(pub static ref [< $key:upper >]: [u8; 32] = get_prefixed_db_key($value);)*
            }

            #[allow(non_snake_case)]
            #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
            pub struct EthDatabaseKeysJson {
                $([< $key:upper >]: String,)*
            }

            impl EthDatabaseKeysJson {
                pub fn new() -> Self {
                    Self {
                        $([< $key:upper >]: hex::encode(&*[< $key:upper >]),)*
                    }
                }
            }
        }
    }
}

create_eth_db_keys!(
    "ETH_CHAIN_ID_KEY" => "eth-chain-id",
    "ETH_GAS_PRICE_KEY" => "eth-gas-price",
    "ETH_ADDRESS_KEY" => "eth-address-key",
    "ETH_LINKER_HASH_KEY" => "linker-hash-key",
    "ETH_ACCOUNT_NONCE_KEY" => "eth-account-nonce",
    "ETH_ANY_SENDER_NONCE_KEY" => "any-sender-nonce",
    "ETH_PRIVATE_KEY_DB_KEY" => "eth-private-key-key",
    "ETH_PTOKEN_GENESIS_HASH_KEY" => "provable-ptoken",
    "ETH_CANON_BLOCK_HASH_KEY" => "canon-block-hash-key",
    "ETH_ANCHOR_BLOCK_HASH_KEY" => "anchor-block-hash-key",
    "ETH_LATEST_BLOCK_HASH_KEY" => "latest-block-hash-key",
    "ETH_TAIL_BLOCK_HASH_KEY" => "eth-tail-block-hash-key",
    "ETH_CANON_TO_TIP_LENGTH_KEY" => "canon-to-tip-length-key",
    "ETH_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY" => "eth-smart-contract",
    "ETH_ERC777_PROXY_CONTACT_ADDRESS_KEY" => "erc-777-proxy-contract-address-key",
    "ETH_ROUTER_SMART_CONTRACT_ADDRESS_KEY" => "eth-router-smart-contract-address-key",
    "ETH_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY" => "eos-on-eth-smart-contract-address-key",
    "ETH_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY" => "erc20-on-eos-smart-contract-address-key",
    "ETH_INT_ON_EVM_SMART_CONTRACT_ADDRESS_KEY" => "eth-int-on-evm-smart-contract-address-key",
    "ETH_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY" => "erc20-on-evm-eth-smart-contract-address-key"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eth_database_keys_should_stay_consistent() {
        #[rustfmt::skip]
        let expected_result = EthDatabaseKeysJson {
            ETH_ANY_SENDER_NONCE_KEY:
                "09feb18750877b8b216cf9dc0bf587dfc4d043620252e1a7a33353710939c2ae".to_string(),
            ETH_BTC_ON_ETH_SMART_CONTRACT_ADDRESS_KEY:
                "f2289049ab0275224d98f6f7d6b2e5c0b301167d04b83aa724024fcad81d61fc".to_string(),
            ETH_EOS_ON_ETH_SMART_CONTRACT_ADDRESS_KEY:
                "13a27c2fe10330e66ea6c562272bcbef4e7ebd003aed087dba387ac43a7f5fd4".to_string(),
            ETH_ERC20_ON_EOS_SMART_CONTRACT_ADDRESS_KEY:
                "fb2788804c9b7b8c40b191f4da2e4db2602a2f1deaaefc052bf1d38220db1dcf".to_string(),
            ETH_ERC20_ON_EVM_SMART_CONTRACT_ADDRESS_KEY:
                "7709f182e4be2554442ffb3637f3417dd75cef4ccb13942d2e35c5d6ace6c503".to_string(),
            ETH_ERC777_PROXY_CONTACT_ADDRESS_KEY:
                "a2e7337756b00998e6efd72220477f4de76ceac441298d6770fff827837b27a6".to_string(),
            ETH_ACCOUNT_NONCE_KEY:
                "713a7d7396c523b7978cd822839e0186395053745941615b0370c0bb72b4dcf4".to_string(),
            ETH_ADDRESS_KEY:
                "bfd203dc3411da4e18d157e87b94507a428060618fcf3163357a1fabe93fba1a".to_string(),
            ETH_ANCHOR_BLOCK_HASH_KEY:
                "1087f2e9bfa897df4da210822cc94bcf77ee11396cf9d3cd247b06aeeb289737".to_string(),
            ETH_CANON_BLOCK_HASH_KEY:
                "c737daae274d21e37403be7d3d562c493332c381ee2b0f3fa0b2286af8b8e5c2".to_string(),
            ETH_CANON_TO_TIP_LENGTH_KEY:
                "192b7e4da694bf96fbc089656a3ba0f63f6263a95af257b693e8dee84334b38c".to_string(),
            ETH_CHAIN_ID_KEY:
                "47199e3b0ffc301baeedd4eb87ebf5ef3829496c8ab2660a6038a62e36e9222f".to_string(),
            ETH_GAS_PRICE_KEY:
                "ecf932d3aca97f12884bc42af7607469feba2206e8b1d37ed1328d477c747346".to_string(),
            ETH_LATEST_BLOCK_HASH_KEY:
                "8b39bef2b5b1e9564bb4a60c8211c32e2f94dc88cae8cfbaad42b2e7e527ea7a".to_string(),
            ETH_INT_ON_EVM_SMART_CONTRACT_ADDRESS_KEY:
                "a1552e7ee400c2adf873879fc3efefea72db11307ad3c873506e1f3be8fd31db".to_string(),
            ETH_LINKER_HASH_KEY:
                "1c045b32a91a460a8a210de0a9b757da8fc21844f02399b558c3c87917122b58".to_string(),
            ETH_PRIVATE_KEY_DB_KEY:
                "eec538cafefe65e094e2e70364da2f2f6e752209e1974e38a9b23ca8ce22b73d".to_string(),
            ETH_TAIL_BLOCK_HASH_KEY:
                "539205e110a233c64f983acf425f1d2cf6cb6535a0241a3722a512690eeba758".to_string(),
            ETH_PTOKEN_GENESIS_HASH_KEY:
                "7eb2e65416dd107602495454d1ed094ae475cff2f3bfb2e2ae68a1c52bc0d66f".to_string(),
            ETH_ROUTER_SMART_CONTRACT_ADDRESS_KEY:
                "7e4ba9ad69fafede39d72a5e5d05953c4261d16ede043978031bc425d2e3b1d2".to_string(),
        };
        let result = EthDatabaseKeysJson::new();
        assert_eq!(result, expected_result)
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
