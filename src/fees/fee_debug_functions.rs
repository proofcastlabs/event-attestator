use serde_json::json;

use crate::{
    check_debug_mode::check_debug_mode,
    fees::{fee_database_utils::FeeDatabaseUtils, fee_utils::sanity_check_basis_points_value},
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

/// # Debug Put BTC-on-ETH Peg-In Basis-Points In DB
///
/// This function sets to the given value the `BTC-on-ETH` peg-in basis-points in the encrypted
/// database.
pub fn debug_put_btc_on_eth_peg_in_basis_points_in_db<D: DatabaseInterface>(
    db: &D,
    basis_points: u64,
) -> Result<String> {
    info!("✔ Debug setting `BTC-on-ETh` peg-in basis-points to {}", basis_points);
    check_debug_mode()
        .and_then(|_| sanity_check_basis_points_value(basis_points))
        .and_then(|_| db.start_transaction())
        .and_then(|_| FeeDatabaseUtils::new_for_btc_on_eth().put_peg_in_basis_points_in_db(db, basis_points))
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"set_btc_on_eth_peg_in_basis_points":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Put BTC-on-ETH Peg-Out Basis-Points In DB
///
/// This function sets to the given value the `BTC-on-ETH` peg-out basis-points in the encrypted
/// database.
pub fn debug_put_btc_on_eth_peg_out_basis_points_in_db<D: DatabaseInterface>(
    db: &D,
    basis_points: u64,
) -> Result<String> {
    info!("✔ Debug setting `BTC-on-ETH` peg-out basis-points to {}", basis_points);
    check_debug_mode()
        .and_then(|_| sanity_check_basis_points_value(basis_points))
        .and_then(|_| db.start_transaction())
        .and_then(|_| FeeDatabaseUtils::new_for_btc_on_eth().put_peg_out_basis_points_in_db(db, basis_points))
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"set_btc_on_eth_peg_out_basis_points":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{errors::AppError, fees::fee_constants::MAX_FEE_BASIS_POINTS};

    #[test]
    fn should_pass_basis_points_sanity_check() {
        let basis_points = MAX_FEE_BASIS_POINTS - 1;
        let result = sanity_check_basis_points_value(basis_points).unwrap();
        assert_eq!(result, basis_points)
    }

    #[test]
    fn should_fail_basis_points_sanity_check() {
        let expected_err = format!("Error! Basis points exceeds maximum of {}!", MAX_FEE_BASIS_POINTS);
        let basis_points = MAX_FEE_BASIS_POINTS + 1;
        match sanity_check_basis_points_value(basis_points) {
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Ok(_) => panic!("Should not have succeeded!"),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
