use crate::{
    types::Result,
    traits::DatabaseInterface,
    btc_on_eos::btc::btc_database_utils::put_btc_account_nonce_in_db,
};

pub fn increment_signature_nonce<D>(
    db: &D,
    current_nonce: u64,
    num_signatures: u64,
) -> Result<()>
    where D: DatabaseInterface
{
    debug!("✔ Incrementing signature nonce from {} to {}", current_nonce, num_signatures + current_nonce);
    put_btc_account_nonce_in_db(db, current_nonce + num_signatures)
}
