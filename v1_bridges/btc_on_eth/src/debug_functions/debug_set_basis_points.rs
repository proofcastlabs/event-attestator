use common::{
    fees::{fee_database_utils::FeeDatabaseUtils, fee_utils::sanity_check_basis_points_value},
    traits::DatabaseInterface,
    types::Result,
    utils::prepend_debug_output_marker_to_string,
};
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
use serde_json::json;

use crate::constants::CORE_TYPE;

#[named]
fn debug_put_btc_on_eth_basis_points_in_db<D: DatabaseInterface>(
    db: &D,
    basis_points: u64,
    is_peg_in: bool,
    signature: &str,
) -> Result<String> {
    let suffix = if is_peg_in { "in" } else { "out" };
    info!(
        "✔ Debug setting `BtcOnEth` peg-{} basis-points to {}",
        suffix, basis_points
    );
    db.start_transaction()
        .and_then(|_| get_debug_command_hash!(function_name!(), &basis_points, &is_peg_in)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| sanity_check_basis_points_value(basis_points))
        .and_then(|_| {
            if is_peg_in {
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
) -> Result<String> {
    info!("✔ Debug setting `BTC-on-ETH` peg-in basis-points to {}", basis_points);
    debug_put_btc_on_eth_basis_points_in_db(db, basis_points, true, signature)
}

/// # Debug Put BTC-on-ETH Peg-Out Basis-Points In DB
///
/// This function sets to the given value the `BTC-on-ETH` peg-out basis-points in the encrypted
/// database.
pub fn debug_put_btc_on_eth_peg_out_basis_points_in_db<D: DatabaseInterface>(
    db: &D,
    basis_points: u64,
    signature: &str,
) -> Result<String> {
    info!("✔ Debug setting `BTC-on-ETH` peg-out basis-points to {}", basis_points);
    debug_put_btc_on_eth_basis_points_in_db(db, basis_points, false, signature)
}
