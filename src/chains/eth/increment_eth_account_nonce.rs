use crate::{chains::eth::eth_database_utils_redux::EthDatabaseUtils, traits::DatabaseInterface, types::Result};

pub fn increment_eth_account_nonce<D: DatabaseInterface>(
    eth_db_utils: &EthDatabaseUtils<D>,
    current_nonce: u64,
    num_signatures: u64,
) -> Result<()> {
    let new_nonce = num_signatures + current_nonce;
    info!(
        "✔ Incrementing ETH account nonce by {} from {} to {}",
        num_signatures, current_nonce, new_nonce
    );
    eth_db_utils.put_eth_account_nonce_in_db(new_nonce)
}
