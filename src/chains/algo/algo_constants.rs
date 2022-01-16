pub use serde_json::{json, Value as JsonValue};

use crate::utils::get_prefixed_db_key;

pub const ALGO_CORE_IS_INITIALIZED_JSON: &str = "{algo_core_initialized:true}";

pub fn get_algo_constants_db_keys() -> JsonValue {
    json!({
        "ALGO_REDEEM_ADDRESS_KEY":
            hex::encode(ALGO_REDEEM_ADDRESS_KEY.to_vec()),
        "ALGO_GENESIS_HASH_KEY":
            hex::encode(ALGO_GENESIS_HASH_KEY.to_vec()),
        "ALGO_LATEST_BLOCK_NUMBER_KEY":
            hex::encode(ALGO_LATEST_BLOCK_NUMBER_KEY.to_vec()),
    })
}

lazy_static! {
    pub static ref ALGO_GENESIS_HASH_KEY: [u8; 32] = get_prefixed_db_key("algo-genesis-hash-key");
    pub static ref ALGO_REDEEM_ADDRESS_KEY: [u8; 32] = get_prefixed_db_key("algo-redeem-address-key");
    pub static ref ALGO_LATEST_BLOCK_NUMBER_KEY: [u8; 32] = get_prefixed_db_key("algo-latest-block-number-key");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_all_algo_db_keys() {
        let expected_result = json!({
            "ALGO_REDEEM_ADDRESS_KEY":
                "d3e2a66c27b833ffbf3049d9bc64c15a32a368d5998b57e127edfdf213969668",
            "ALGO_GENESIS_HASH_KEY":
                "9fe966baf0d59f44daee886cad2eef74e47ca48a76e83030f1b526bf49e5d02d",
            "ALGO_LATEST_BLOCK_NUMBER_KEY":
                "b2eb58d1d1b1b7300a8d61e2cd11d0ce82583e8a2191b32b9d87adbf01d430eb",
        });
        let result = get_algo_constants_db_keys();
        assert_eq!(result, expected_result)
    }
}
