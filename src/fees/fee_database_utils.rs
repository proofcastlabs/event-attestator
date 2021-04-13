use crate::{
    database_utils::{get_u64_from_db, put_u64_in_db},
    fees::fee_constants::{
        BTC_ON_ETH_ACCRUED_FEES_KEY,
        BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY,
        BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY,
    },
    traits::DatabaseInterface,
    types::Result,
};

pub fn get_btc_on_eth_accrued_fees_from_db<D: DatabaseInterface>(db: &D) -> Result<u64> {
    debug!("✔ Getting BTC accrued fees from db...");
    get_u64_from_db(db, &BTC_ON_ETH_ACCRUED_FEES_KEY.to_vec()).or_else(|_| {
        debug!("✔ No `BTC_ON_ETH_ACCRUED_FEES_KEY` value set in db, defaulting to 0!");
        Ok(0)
    })
}

pub fn get_btc_on_eth_peg_in_basis_points_from_db<D: DatabaseInterface>(db: &D) -> Result<u64> {
    debug!("✔ Getting `BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY` from db...");
    get_u64_from_db(db, &BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY.to_vec()).or_else(|_| {
        debug!("✔ No `BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY` value set in db, defaulting to 0!");
        Ok(0)
    })
}

pub fn get_btc_on_eth_peg_out_basis_points_from_db<D: DatabaseInterface>(db: &D) -> Result<u64> {
    debug!("✔ Getting `BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY` from db...");
    get_u64_from_db(db, &BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY.to_vec()).or_else(|_| {
        debug!("✔ No `BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY` value set in db, defaulting to 0!");
        Ok(0)
    })
}

pub fn put_btc_on_eth_peg_in_basis_points_in_db<D: DatabaseInterface>(db: &D, basis_points: u64) -> Result<()> {
    debug!("✔ Putting `BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY` of {} in db...", basis_points);
    put_u64_in_db(db, &BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY.to_vec(), basis_points)
}

pub fn put_btc_on_eth_peg_out_basis_points_in_db<D: DatabaseInterface>(db: &D, basis_points: u64) -> Result<()> {
    debug!("✔ Putting `BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY` of {} in db...", basis_points);
    put_u64_in_db(db, &BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY.to_vec(), basis_points)
}

fn put_btc_on_eth_accrued_fees_in_db<D: DatabaseInterface>(db: &D, amount: u64) -> Result<()> {
    debug!("✔ Putting BTC accrued fees in db...");
    put_u64_in_db(db, &BTC_ON_ETH_ACCRUED_FEES_KEY.to_vec(), amount)
}

pub fn increment_btc_on_eth_accrued_fees<D: DatabaseInterface>(db: &D, increment_amount: u64) -> Result<()> {
    debug!("✔ Incrementing BTC accrued fees in db...");
    get_btc_on_eth_accrued_fees_from_db(db).and_then(|accrued_fees| {
        let total_after_incrementing = accrued_fees + increment_amount;
        debug!("✔ Accrued fees before incrementing: {}", accrued_fees);
        debug!("✔           Incrementing by amount: {}", increment_amount);
        debug!("✔        Total after incremeneting: {}", total_after_incrementing);
        put_btc_on_eth_accrued_fees_in_db(db, total_after_incrementing)
    })
}
