pub use serde_json::{json, Value as JsonValue};

use crate::utils::get_prefixed_db_key;

pub fn get_fee_constants_db_keys() -> JsonValue {
    json!({
        "BTC_ON_ETH_ACCRUED_FEES_KEY": hex::encode(BTC_ON_ETH_ACCRUED_FEES_KEY.to_vec()),
        "BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY": hex::encode(BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY.to_vec()),
        "BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY": hex::encode(BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY.to_vec()),
    })
}

lazy_static! {
    pub static ref BTC_ON_ETH_ACCRUED_FEES_KEY: [u8; 32] = get_prefixed_db_key("btc-on-eth-accrued-fees-key");
    pub static ref BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY: [u8; 32] =
        get_prefixed_db_key("btc-on-eth-peg-in-basis-points-key");
    pub static ref BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY: [u8; 32] =
        get_prefixed_db_key("btc-on-eth-peg-out-basis-points-key");
}
