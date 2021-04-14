use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};

use crate::{
    fees::{
        fee_constants::{BTC_ON_ETH_ACCRUED_FEES_KEY, DISABLE_FEES},
        fee_database_utils::{
            get_btc_on_eth_accrued_fees_from_db,
            get_btc_on_eth_last_fee_withdrawal_timestamp_from_db,
            get_btc_on_eth_peg_in_basis_points_from_db,
            get_btc_on_eth_peg_out_basis_points_from_db,
        },
    },
    traits::DatabaseInterface,
    types::Result,
    utils::convert_unix_timestamp_to_human_readable,
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
    fn get_last_withdrawal_date_as_human_readable_string(timestamp: u64) -> String {
        if timestamp == 0 {
            "Fees have not yet been withdrawn!".to_string()
        } else {
            convert_unix_timestamp_to_human_readable(timestamp)
        }
    }

    pub fn new_for_btc_on_eth<D: DatabaseInterface>(db: &D) -> Result<Self> {
        Ok(Self::new(vec![FeeStateForToken {
            token_symbol: "BTC".to_string(),
            accrued_fees: get_btc_on_eth_accrued_fees_from_db(db)?,
            accrued_fees_db_key: hex::encode(*BTC_ON_ETH_ACCRUED_FEES_KEY),
            peg_in_basis_points: get_btc_on_eth_peg_in_basis_points_from_db(db)?,
            peg_out_basis_points: get_btc_on_eth_peg_out_basis_points_from_db(db)?,
            last_withdrawal: Self::get_last_withdrawal_date_as_human_readable_string(
                get_btc_on_eth_last_fee_withdrawal_timestamp_from_db(db)?,
            ),
        }]))
    }
}
