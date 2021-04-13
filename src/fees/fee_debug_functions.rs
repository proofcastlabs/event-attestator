use serde_json::json;

use crate::{
    check_debug_mode::check_debug_mode,
    fees::fee_database_utils::{put_btc_on_eth_peg_in_basis_points_in_db, put_btc_on_eth_peg_out_basis_points_in_db},
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
        .and_then(|_| db.start_transaction())
        .and_then(|_| put_btc_on_eth_peg_in_basis_points_in_db(db, basis_points))
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
    info!("✔ Debug setting `BTC-on-ETh` peg-out basis-points to {}", basis_points);
    check_debug_mode()
        .and_then(|_| db.start_transaction())
        .and_then(|_| put_btc_on_eth_peg_out_basis_points_in_db(db, basis_points))
        .and_then(|_| db.end_transaction())
        .and(Ok(json!({"set_btc_on_eth_peg_out_basis_points":true}).to_string()))
        .map(prepend_debug_output_marker_to_string)
}
