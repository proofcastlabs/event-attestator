use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};

use crate::{
    fees::{
        fee_constants::{BTC_ON_ETH_FEE_DB_KEYS, DISABLE_FEES},
        fee_database_utils::FeeDatabaseUtils,
        fee_utils::get_last_withdrawal_date_as_human_readable_string,
    },
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
pub struct FeesEnclaveState {
    fees_enabled: bool,
    fees: FeeStateForTokens,
}

impl FeesEnclaveState {
    pub fn new_for_btc_on_eth<D: DatabaseInterface>(db: &D) -> Result<Self> {
        Ok(Self {
            fees_enabled: !DISABLE_FEES,
            fees: FeeStateForTokens::new_for_btc_on_eth(db)?,
        })
    }
}

#[derive(Serialize, Deserialize)]
pub struct FeeStateForToken {
    token_symbol: String,
    accrued_fees: u64,
    peg_in_basis_points: u64,
    peg_out_basis_points: u64,
    accrued_fees_db_key: String,
    last_withdrawal: String,
}

#[derive(Serialize, Deserialize, Deref, Constructor)]
pub struct FeeStateForTokens(Vec<FeeStateForToken>);

impl FeeStateForTokens {
    pub fn new_for_btc_on_eth<D: DatabaseInterface>(db: &D) -> Result<Self> {
        Ok(Self::new(vec![FeeStateForToken {
            token_symbol: "BTC".to_string(),
            accrued_fees: FeeDatabaseUtils::new_for_btc_on_eth().get_accrued_fees_from_db(db)?,
            accrued_fees_db_key: hex::encode(&BTC_ON_ETH_FEE_DB_KEYS.accrued_fees_db_key),
            peg_in_basis_points: FeeDatabaseUtils::new_for_btc_on_eth().get_peg_in_basis_points_from_db(db)?,
            peg_out_basis_points: FeeDatabaseUtils::new_for_btc_on_eth().get_peg_out_basis_points_from_db(db)?,
            last_withdrawal: get_last_withdrawal_date_as_human_readable_string(
                FeeDatabaseUtils::new_for_btc_on_eth().get_last_fee_withdrawal_timestamp_from_db(db)?,
            ),
        }]))
    }
}
