use common::{core_type::CoreType, traits::DatabaseInterface, types::Result};
use common_algorand::AlgoDbUtils;
use common_debug_signers::validate_debug_command_signature;
use function_name::named;
use rust_algorand::{AlgorandAddress, AlgorandHash, AlgorandKeys, AlgorandTransaction, MicroAlgos};
use serde_json::json;

use crate::constants::CORE_TYPE;

fn get_application_opt_in_tx_hex(
    app_id: u64,
    fee: &MicroAlgos,
    first_valid_round: u64,
    sender: &AlgorandAddress,
    genesis_hash: &AlgorandHash,
    private_key: &AlgorandKeys,
) -> Result<String> {
    Ok(
        AlgorandTransaction::application_opt_in(app_id, fee, first_valid_round, sender, genesis_hash, None)?
            .sign(private_key)?
            .to_hex()?,
    )
}

/// # Opt In To Application
///
/// This function creates an application-opt-in transaction for the core's Algorand account. Once
/// broadcast, this transaction allows the core's account to interact with that application.
/// The function requires a first-valid-round parameter to be passed in which defines whence
/// the transaction is broadcastable.
#[named]
pub fn debug_opt_in_to_application<D: DatabaseInterface>(
    db: &D,
    app_id: u64,
    first_valid_round: u64,
    signature: &str,
) -> Result<String> {
    info!("✔ Opting in to ALGO asset...");
    let algo_db_utils = AlgoDbUtils::new(db);
    db.start_transaction()
        .and_then(|_| CoreType::check_is_initialized(db))
        .and_then(|_| get_debug_command_hash!(function_name!(), &app_id, &first_valid_round)())
        .and_then(|hash| validate_debug_command_signature(db, &CORE_TYPE, signature, &hash, cfg!(test)))
        .and_then(|_| {
            get_application_opt_in_tx_hex(
                app_id,
                &algo_db_utils.get_algo_fee()?,
                first_valid_round,
                &algo_db_utils.get_redeem_address()?,
                &algo_db_utils.get_genesis_hash()?,
                &algo_db_utils.get_algo_private_key()?,
            )
        })
        .and_then(|signed_tx_hex| {
            db.end_transaction()?;
            Ok(json!({ "success": true, "tx": signed_tx_hex }).to_string())
        })
}
