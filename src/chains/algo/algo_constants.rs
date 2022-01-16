use std::fmt;

use paste::paste;
use serde::{Deserialize, Serialize};
pub use serde_json::{json, Value as JsonValue};

use crate::{types::Result, utils::get_prefixed_db_key};

pub const ALGO_CORE_IS_INITIALIZED_JSON: &str = "{algo_core_initialized:true}";

macro_rules! write_algo_db_keys {
    ($($name:expr),*) => {
        paste! {
            lazy_static! {
                $(pub static ref [< $name:snake:upper >]: [u8; 32] = get_prefixed_db_key($name);)*
            }

            // NOTE: This struct actually ends up as SCREAMING_SNAKE_CASE due to
            // the paste! macro's :snake:upper stuff, but the compiler can't figure that out.
            #[allow(non_snake_case)]
            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            pub struct AlgoDbKeysJson {
                $([< $name:snake:upper >]: String,)*
            }

            impl AlgoDbKeysJson {
                fn new() -> Self {
                    Self {
                        $([< $name:snake:upper >]: hex::encode(&*[< $name:snake:upper >]),)*
                    }
                }

                pub fn to_string(&self) -> Result<String> {
                    Ok(serde_json::to_string(self)?)
                }
            }
        }
    }
}

write_algo_db_keys!(
    "algo_fee_key",
    "algo_redeem_address_key",
    "algo_tail_block_hash_key",
    "algo_canon_block_hash_key",
    "algo_anchor_block_hash_key",
    "algo_latest_block_hash_key",
    "algo_genesis_block_hash_key",
    "algo_latest_block_number_key",
    "algo_canon_to_tip_length_key"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algo_db_keys_should_remain_consistent() {
        let expected_result = AlgoDbKeysJson {
            ALGO_FEE_KEY: "d284e359e0a2076c909ee55d8deaf1e05b5488a997f18bf86e0928c4fbc5c638".to_string(),
            ALGO_REDEEM_ADDRESS_KEY: "6e4a528af852818a2f5c1660679873fbe3a49ab57ecf14bf0f542220e95cc6d4".to_string(),
            ALGO_TAIL_BLOCK_HASH_KEY: "2a307fe54ac8b580e12772152a6be38285afb11a932ab817c423a580c474fb3f".to_string(),
            ALGO_CANON_BLOCK_HASH_KEY: "1a4b2db39e866baa1e76f114c6620a94e7cd078bf1c81f5cd286e4213ea60892".to_string(),
            ALGO_ANCHOR_BLOCK_HASH_KEY: "0708c1e329a262c9ce0e39d91a05be6dbb270861869b2c48d8aa4d8e7aa58c75".to_string(),
            ALGO_LATEST_BLOCK_HASH_KEY: "d5743e9bee45679ce65bf04dc3fbce27ef1f148a13a37e4234288f92d3e2e124".to_string(),
            ALGO_GENESIS_BLOCK_HASH_KEY: "e10b845e685c345196e1b4f41a91fa74fc8ae7f000184f222f4b5df649b50585".to_string(),
            ALGO_LATEST_BLOCK_NUMBER_KEY: "c3c70374f7eeb4892998285bf504943fcac222a6df561247c8a53b108ef9556d"
                .to_string(),
            ALGO_CANON_TO_TIP_LENGTH_KEY: "295dafb37cf7d99e712b44c066951b962bef0243abb56b5aba1172ea70bfb5f5"
                .to_string(),
        };
        let result = AlgoDbKeysJson::new();
        assert_eq!(result, expected_result)
    }
}
