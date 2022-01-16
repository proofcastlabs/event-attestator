pub use serde_json::{json, Value as JsonValue};

use crate::utils::get_prefixed_db_key;

pub const ALGO_CORE_IS_INITIALIZED_JSON: &str = "{algo_core_initialized:true}";

// TODO use a macro to generate all of these quickly.

pub fn get_algo_constants_db_keys() -> JsonValue {
    json!({
        "ALGO_REDEEM_ADDRESS_KEY":
            hex::encode(ALGO_REDEEM_ADDRESS_KEY.to_vec()),
        "ALGO_GENESIS_BLOCK_HASH_KEY":
            hex::encode(ALGO_GENESIS_BLOCK_HASH_KEY.to_vec()),
        "ALGO_LATEST_BLOCK_NUMBER_KEY":
            hex::encode(ALGO_LATEST_BLOCK_NUMBER_KEY.to_vec()),
        "ALGO_TAIL_BLOCK_HASH_KEY":
            hex::encode(ALGO_TAIL_BLOCK_HASH_KEY.to_vec()),
        "ALGO_ANCHOR_BLOCK_HASH_KEY":
            hex::encode(ALGO_ANCHOR_BLOCK_HASH_KEY.to_vec()),
        "ALGO_CANON_BLOCK_HASH_KEY":
            hex::encode(ALGO_CANON_BLOCK_HASH_KEY.to_vec()),
        "ALGO_LATEST_BLOCK_HASH_KEY":
            hex::encode(ALGO_LATEST_BLOCK_HASH_KEY.to_vec()),
    })
}

lazy_static! {
    pub static ref ALGO_REDEEM_ADDRESS_KEY: [u8; 32] = get_prefixed_db_key("algo-redeem-address-key");
    pub static ref ALGO_TAIL_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("algo-tail-block-hash-key");
    pub static ref ALGO_CANON_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("algo-canon-block-hash-key");
    pub static ref ALGO_ANCHOR_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("algo-anchor-block-hash-key");
    pub static ref ALGO_LATEST_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("algo-latest-block-hash-key");
    pub static ref ALGO_GENESIS_BLOCK_HASH_KEY: [u8; 32] = get_prefixed_db_key("algo-genesis-block-hash-key");
    pub static ref ALGO_LATEST_BLOCK_NUMBER_KEY: [u8; 32] = get_prefixed_db_key("algo-latest-block-number-key");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_all_algo_db_keys() {
        let expected_result = json!({
            "ALGO_ANCHOR_BLOCK_HASH_KEY":
                "309d45a2c467a0206c79cfea653244f1a430bd2b35ef4d4c93b9810fa6edccdf",
            "ALGO_CANON_BLOCK_HASH_KEY":
                "0dc734a3a3f99f38a38d3d01e324f8395681e30c653891b5744542eb10b38256",
            "ALGO_GENESIS_BLOCK_HASH_KEY":
                "bbd97d1a3028c1b5110ad13d50c544908b9c5ce3203bce409510d0aaaf85981a",
            "ALGO_LATEST_BLOCK_HASH_KEY":
                "b4304dae5b447bd29e015334d65946e519e34fe604acb07a7cf2703f5f5f50ca",
            "ALGO_LATEST_BLOCK_NUMBER_KEY":
                "b2eb58d1d1b1b7300a8d61e2cd11d0ce82583e8a2191b32b9d87adbf01d430eb",
            "ALGO_REDEEM_ADDRESS_KEY":
                "d3e2a66c27b833ffbf3049d9bc64c15a32a368d5998b57e127edfdf213969668",
            "ALGO_TAIL_BLOCK_HASH_KEY":
                "c00de4ca233a9fc03ddbfe85125af5064fb071cf75800d06dd4e07e2fac13747",
        });
        let result = get_algo_constants_db_keys();
        assert_eq!(result, expected_result)
    }
}
