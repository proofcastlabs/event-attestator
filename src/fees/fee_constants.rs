pub use serde_json::{json, Value as JsonValue};

use crate::{core_type::CoreType, types::Bytes, utils::get_prefixed_db_key};

#[cfg(not(feature = "disable-fees"))]
pub const DISABLE_FEES: bool = false;

#[cfg(feature = "disable-fees")]
pub const DISABLE_FEES: bool = true;

pub const MAX_FEE_BASIS_POINTS: u64 = 1000;

lazy_static! {
    pub static ref BTC_ON_ETH_ACCRUED_FEES_KEY: [u8; 32] = get_prefixed_db_key("btc-on-eth-accrued-fees-key");
    pub static ref BTC_ON_ETH_LAST_FEE_WITHDRAWAL_TIMESTAMP_KEY: [u8; 32] =
        get_prefixed_db_key("btc-on-eth-last-fee-withdrawal-timestamp");
    pub static ref BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY: [u8; 32] =
        get_prefixed_db_key("btc-on-eth-peg-in-basis-points-key");
    pub static ref BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY: [u8; 32] =
        get_prefixed_db_key("btc-on-eth-peg-out-basis-points-key");
    pub static ref BTC_ON_EOS_ACCRUED_FEES_KEY: [u8; 32] = get_prefixed_db_key("btc-on-eos-accrued-fees-key");
    pub static ref BTC_ON_EOS_LAST_FEE_WITHDRAWAL_TIMESTAMP_KEY: [u8; 32] =
        get_prefixed_db_key("btc-on-eos-last-fee-withdrawal-timestamp");
    pub static ref BTC_ON_EOS_PEG_IN_BASIS_POINTS_KEY: [u8; 32] =
        get_prefixed_db_key("btc-on-eos-peg-in-basis-points-key");
    pub static ref BTC_ON_EOS_PEG_OUT_BASIS_POINTS_KEY: [u8; 32] =
        get_prefixed_db_key("btc-on-eos-peg-out-basis-points-key");
    pub static ref BTC_ON_ETH_FEE_DB_KEYS: FeeConstants = FeeConstants::new(CoreType::BtcOnEth);
    pub static ref BTC_ON_EOS_FEE_DB_KEYS: FeeConstants = FeeConstants::new(CoreType::BtcOnEos);
}

pub struct FeeConstants {
    pub core_type: CoreType,
    pub accrued_fees_key: Bytes,
    pub peg_in_basis_points: Bytes,
    pub last_fee_withdrawal: Bytes,
    pub peg_out_basis_points: Bytes,
}

impl FeeConstants {
    pub fn new(core_type: CoreType) -> Self {
        match core_type {
            CoreType::BtcOnEth => Self {
                core_type,
                accrued_fees_key: get_prefixed_db_key("btc-on-eth-accrued-fees-key").to_vec(),
                peg_in_basis_points: get_prefixed_db_key("btc-on-eth-last-fee-withdrawal-timestamp").to_vec(),
                last_fee_withdrawal: get_prefixed_db_key("btc-on-eth-peg-in-basis-points-key").to_vec(),
                peg_out_basis_points: get_prefixed_db_key("btc-on-eth-peg-out-basis-points-key").to_vec(),
            },
            CoreType::BtcOnEos => Self {
                core_type,
                accrued_fees_key: get_prefixed_db_key("btc-on-eos-accrued-fees-key").to_vec(),
                peg_in_basis_points: get_prefixed_db_key("btc-on-eos-last-fee-withdrawal-timestamp").to_vec(),
                last_fee_withdrawal: get_prefixed_db_key("btc-on-eos-peg-in-basis-points-key").to_vec(),
                peg_out_basis_points: get_prefixed_db_key("btc-on-eos-peg-out-basis-points-key").to_vec(),
            },
            _ => unimplemented!("`FeeConstants` struct not implemented for core type: {}!", core_type),
        }
    }

    pub fn to_json(&self) -> JsonValue {
        let prefix = self.core_type.to_string();
        json!({
            format!("{}_ACCRUED_FEES_KEY", prefix): hex::encode(&self.accrued_fees_key),
            format!("{}_PEG_IN_BASIS_POINTS_KEY", prefix): hex::encode(&self.peg_in_basis_points),
            format!("{}_PEG_OUT_BASIS_POINTS_KEY", prefix): hex::encode(&self.peg_out_basis_points),
            format!("{}_LAST_FEE_WITHDRAWAL_TIMESTAMP_KEY", prefix): hex::encode(&self.last_fee_withdrawal),
        })
    }
}
