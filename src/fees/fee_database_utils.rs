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
    debug!(
        "✔ Putting `BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY` of {} in db...",
        basis_points
    );
    put_u64_in_db(db, &BTC_ON_ETH_PEG_IN_BASIS_POINTS_KEY.to_vec(), basis_points)
}

pub fn put_btc_on_eth_peg_out_basis_points_in_db<D: DatabaseInterface>(db: &D, basis_points: u64) -> Result<()> {
    debug!(
        "✔ Putting `BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY` of {} in db...",
        basis_points
    );
    put_u64_in_db(db, &BTC_ON_ETH_PEG_OUT_BASIS_POINTS_KEY.to_vec(), basis_points)
}

fn put_btc_on_eth_accrued_fees_in_db<D: DatabaseInterface>(db: &D, amount: u64) -> Result<()> {
    debug!("✔ Putting BTC accrued fee value of {} in db...", amount);
    put_u64_in_db(db, &BTC_ON_ETH_ACCRUED_FEES_KEY.to_vec(), amount)
}

pub fn reset_btc_accrued_fees<D: DatabaseInterface>(db: &D) -> Result<()> {
    put_btc_on_eth_accrued_fees_in_db(db, 0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_test_database;

    #[test]
    fn should_put_and_get_btc_on_eth_peg_in_basis_points_in_db() {
        let basis_points: u64 = 1337;
        let db = get_test_database();
        put_btc_on_eth_peg_in_basis_points_in_db(&db, basis_points).unwrap();
        let result = get_btc_on_eth_peg_in_basis_points_from_db(&db).unwrap();
        assert_eq!(result, basis_points);
    }

    #[test]
    fn should_put_and_get_btc_on_eth_peg_out_basis_points_in_db() {
        let basis_points: u64 = 1337;
        let db = get_test_database();
        put_btc_on_eth_peg_out_basis_points_in_db(&db, basis_points).unwrap();
        let result = get_btc_on_eth_peg_out_basis_points_from_db(&db).unwrap();
        assert_eq!(result, basis_points);
    }

    #[test]
    fn should_put_and_get_btc_on_eth_accrued_fees_in_db() {
        let fees: u64 = 1337;
        let db = get_test_database();
        put_btc_on_eth_accrued_fees_in_db(&db, fees).unwrap();
        let result = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        assert_eq!(result, fees);
    }

    #[test]
    fn get_btc_on_eth_peg_in_basis_points_from_db_should_default_to_zero() {
        let db = get_test_database();
        let expected_result = 0;
        let result = get_btc_on_eth_peg_in_basis_points_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn get_btc_on_eth_peg_out_basis_points_from_db_should_default_to_zero() {
        let db = get_test_database();
        let expected_result = 0;
        let result = get_btc_on_eth_peg_out_basis_points_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn get_btc_on_eth_accrued_fees_from_db_should_default_to_zero() {
        let db = get_test_database();
        let expected_result = 0;
        let result = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_increment_btc_on_eth_accrued_fees_in_db() {
        let db = get_test_database();
        let start_value = 1337;
        let increment_amount = 1;
        put_btc_on_eth_accrued_fees_in_db(&db, start_value).unwrap();
        increment_btc_on_eth_accrued_fees(&db, increment_amount).unwrap();
        let result = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        assert_eq!(result, start_value + increment_amount);
    }

    #[test]
    fn should_reset_btc_accrued_fees() {
        let fees = 1337;
        let db = get_test_database();
        put_btc_on_eth_accrued_fees_in_db(&db, fees).unwrap();
        let fees_in_db_before = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        assert_eq!(fees_in_db_before, fees);
        reset_btc_accrued_fees(&db).unwrap();
        let fees_in_db_after = get_btc_on_eth_accrued_fees_from_db(&db).unwrap();
        assert_eq!(fees_in_db_after, 0)
    }
}
