use serde_json::json;

use crate::{
    core_type::CoreType,
    debug_mode::{check_debug_mode, validate_debug_command_signature},
    fees::{fee_database_utils::FeeDatabaseUtils, fee_utils::sanity_check_basis_points_value},
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};

fn debug_put_btc_on_eth_basis_points_in_db<D: DatabaseInterface>(
    db: &D,
    basis_points: u64,
    peg_in: bool,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    let suffix = if peg_in { "in" } else { "out" };
    info!(
        "✔ Debug setting `BtcOnEth` peg-{} basis-points to {}",
        suffix, basis_points
    );
    db.start_transaction()
        .and_then(|_| check_debug_mode())
        .and_then(|_| validate_debug_command_signature(db, &CoreType::BtcOnEth, signature, debug_command_hash))
        .and_then(|_| sanity_check_basis_points_value(basis_points))
        .and_then(|_| {
            if peg_in {
                FeeDatabaseUtils::new_for_btc_on_eth().put_peg_in_basis_points_in_db(db, basis_points)
            } else {
                FeeDatabaseUtils::new_for_btc_on_eth().put_peg_out_basis_points_in_db(db, basis_points)
            }
        })
        .and_then(|_| db.end_transaction())
        .and(Ok(
            json!({format!("set_btc_on_eth_peg_{}_basis_points", suffix):true}).to_string()
        ))
        .map(prepend_debug_output_marker_to_string)
}

/// # Debug Put BTC-on-ETH Peg-In Basis-Points In DB
///
/// This function sets to the given value the `BTC-on-ETH` peg-in basis-points in the encrypted
/// database.
pub fn debug_put_btc_on_eth_peg_in_basis_points_in_db<D: DatabaseInterface>(
    db: &D,
    basis_points: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug setting `BTC-on-ETH` peg-in basis-points to {}", basis_points);
    debug_put_btc_on_eth_basis_points_in_db(db, basis_points, true, signature, debug_command_hash)
}

/// # Debug Put BTC-on-ETH Peg-Out Basis-Points In DB
///
/// This function sets to the given value the `BTC-on-ETH` peg-out basis-points in the encrypted
/// database.
pub fn debug_put_btc_on_eth_peg_out_basis_points_in_db<D: DatabaseInterface>(
    db: &D,
    basis_points: u64,
    signature: &str,
    debug_command_hash: &str,
) -> Result<String> {
    info!("✔ Debug setting `BTC-on-ETH` peg-out basis-points to {}", basis_points);
    debug_put_btc_on_eth_basis_points_in_db(db, basis_points, false, signature, debug_command_hash)
}
