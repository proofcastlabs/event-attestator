pub use serde_json::{json, Value as JsonValue};

use crate::{core_type::CoreType, types::Bytes, utils::get_prefixed_db_key};

#[cfg(not(feature = "disable-fees"))]
pub const DISABLE_FEES: bool = false;

#[cfg(feature = "disable-fees")]
pub const DISABLE_FEES: bool = true;

pub const MAX_FEE_BASIS_POINTS: u64 = 1000;

lazy_static! {
    pub static ref BTC_ON_ETH_FEE_DB_KEYS: FeeConstantDbKeys = FeeConstantDbKeys::new_for_btc_on_eth();
    pub static ref BTC_ON_EOS_FEE_DB_KEYS: FeeConstantDbKeys = FeeConstantDbKeys::new_for_btc_on_eos();
}

#[derive(Clone)]
pub struct FeeConstantDbKeys {
    pub core_type: CoreType,
    pub accrued_fees_db_key: Bytes,
    pub peg_in_basis_points_db_key: Bytes,
    pub last_fee_withdrawal_db_key: Bytes,
    pub peg_out_basis_points_db_key: Bytes,
}

impl FeeConstantDbKeys {
    pub fn new(core_type: CoreType) -> Self {
        Self {
            core_type,
            accrued_fees_db_key: get_prefixed_db_key(&format!("{}-accrued-fees-key", core_type.as_db_key_prefix()))
                .to_vec(),
            peg_in_basis_points_db_key: get_prefixed_db_key(&format!(
                "{}-peg-in-basis-points-key",
                core_type.as_db_key_prefix()
            ))
            .to_vec(),
            last_fee_withdrawal_db_key: get_prefixed_db_key(&format!(
                "{}-last-fee-withdrawal-timestamp",
                core_type.as_db_key_prefix()
            ))
            .to_vec(),
            peg_out_basis_points_db_key: get_prefixed_db_key(&format!(
                "{}-peg-out-basis-points-key",
                core_type.as_db_key_prefix()
            ))
            .to_vec(),
        }
    }

    pub fn new_for_btc_on_eth() -> Self {
        Self::new(CoreType::BtcOnEth)
    }

    pub fn new_for_btc_on_eos() -> Self {
        Self::new(CoreType::BtcOnEos)
    }

    pub fn to_json(&self) -> JsonValue {
        let prefix = self.core_type.to_string();
        json!({
            format!("{}_ACCRUED_FEES_KEY", prefix): hex::encode(&self.accrued_fees_db_key),
            format!("{}_PEG_IN_BASIS_POINTS_KEY", prefix): hex::encode(&self.peg_in_basis_points_db_key),
            format!("{}_PEG_OUT_BASIS_POINTS_KEY", prefix): hex::encode(&self.peg_out_basis_points_db_key),
            format!("{}_LAST_FEE_WITHDRAWAL_TIMESTAMP_KEY", prefix): hex::encode(&self.last_fee_withdrawal_db_key),
        })
    }
}
