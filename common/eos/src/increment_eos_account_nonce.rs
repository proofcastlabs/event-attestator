use common::{traits::DatabaseInterface, types::Result};

use crate::EosDbUtils;

pub fn increment_eos_account_nonce<D: DatabaseInterface>(
    db_utils: &EosDbUtils<D>,
    current_nonce: u64,
    num_signatures: u64,
) -> Result<()> {
    let new_nonce = num_signatures + current_nonce;
    info!(
        "✔ Incrementing eos account nonce by {} from {} to {}",
        num_signatures, current_nonce, new_nonce
    );
    db_utils.put_eos_account_nonce_in_db(new_nonce)
}
