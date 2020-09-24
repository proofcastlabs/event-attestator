use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::btc::btc_database_utils::put_btc_account_nonce_in_db,
};

pub fn increment_btc_account_nonce<D>(
    db: &D,
    current_nonce: u64,
    num_signatures: u64,
) -> Result<()>
    where D: DatabaseInterface
{
    let new_nonce = num_signatures + current_nonce;
    info!("✔ Incrementing btc account nonce by {} nonce from {} to {}", num_signatures, current_nonce, new_nonce);
    put_btc_account_nonce_in_db(db, new_nonce)
}

