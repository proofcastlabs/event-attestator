use serde::{Deserialize, Serialize};

use crate::{
    fees::{
        fee_constants::BTC_ON_ETH_ACCRUED_FEES_KEY,
        fee_database_utils::{
            get_btc_on_eth_accrued_fees_from_db,
            get_btc_on_eth_peg_in_basis_points_from_db,
            get_btc_on_eth_peg_out_basis_points_from_db,
        },
    },
    traits::DatabaseInterface,
    types::Result,
};

#[derive(Serialize, Deserialize)]
pub struct FeeEnclaveState {
    accrued_fees: u64,
    peg_in_basis_points: u64,
    peg_out_basis_points: u64,
    accrued_fees_db_key: String,
}

impl FeeEnclaveState {
    pub fn new_for_btc_on_eth<D: DatabaseInterface>(db: &D) -> Result<Self> {
        Ok(Self {
            accrued_fees: get_btc_on_eth_accrued_fees_from_db(db)?,
            accrued_fees_db_key: hex::encode(*BTC_ON_ETH_ACCRUED_FEES_KEY),
            peg_in_basis_points: get_btc_on_eth_peg_in_basis_points_from_db(db)?,
            peg_out_basis_points: get_btc_on_eth_peg_out_basis_points_from_db(db)?,
        })
    }
}
