use rust_algorand::{AlgorandAddress, AlgorandHash, AlgorandKeys, AlgorandTransaction, MicroAlgos};
use serde_json::json;

use crate::{
    chains::{algo::algo_database_utils::AlgoDbUtils, eth::eth_database_utils::EthDbUtils},
    int_on_algo::check_core_is_initialized::check_core_is_initialized,
    traits::DatabaseInterface,
    types::Result,
};

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
pub fn opt_in_to_application<D: DatabaseInterface>(db: &D, app_id: u64, first_valid_round: u64) -> Result<String> {
    info!("✔ Opting in to ALGO asset...");
    let int_db_utils = EthDbUtils::new(db);
    let algo_db_utils = AlgoDbUtils::new(db);
    check_core_is_initialized(&int_db_utils, &algo_db_utils)
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
        .map(|signed_tx_hex| json!({ "success": true, "tx": signed_tx_hex }).to_string())
}
