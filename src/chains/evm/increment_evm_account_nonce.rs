use crate::{chains::eth::eth_database_utils::EthDatabaseUtils, traits::DatabaseInterface, types::Result};

pub fn increment_evm_account_nonce<D: DatabaseInterface>(
    db: &D,
    current_nonce: u64,
    num_signatures: u64,
) -> Result<()> {
    let new_nonce = num_signatures + current_nonce;
    info!(
        "✔ Incrementing EVM account nonce by {} from {} to {}",
        num_signatures, current_nonce, new_nonce
    );
    EthDatabaseUtils::new_for_evm(db).put_eth_account_nonce_in_db(new_nonce) // TODO pass in db utils!
}
