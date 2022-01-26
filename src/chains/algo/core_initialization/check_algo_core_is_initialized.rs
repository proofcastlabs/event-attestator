use crate::{chains::algo::algo_database_utils::AlgoDbUtils, traits::DatabaseInterface, types::Result};

fn is_algo_core_initialized<D: DatabaseInterface>(db_utils: &AlgoDbUtils<D>) -> bool {
    db_utils.get_redeem_address().is_ok()
}

pub fn check_algo_core_is_initialized<D: DatabaseInterface>(db_utils: &AlgoDbUtils<D>) -> Result<()> {
    info!("✔ Checking ALGO core is initialized...");
    if is_algo_core_initialized(db_utils) {
        Ok(())
    } else {
        Err("ALGO core not initialized!".into())
    }
}
