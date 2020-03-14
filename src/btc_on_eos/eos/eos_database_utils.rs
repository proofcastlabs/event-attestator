use std::str::FromStr;
use eos_primitives::{
    Checksum256,
    AccountName as EosAccountName,
};
use crate::btc_on_eos::{
    errors::AppError,
    traits::DatabaseInterface,
    utils::convert_hex_to_checksum256,
    types::{
        Bytes,
        Result,
    },
    database_utils::{
        put_u64_in_db,
        get_u64_from_db,
        put_string_in_db,
        get_string_from_db,
    },
    eos::{
        eos_state::EosState,
        eos_crypto::eos_private_key::EosPrivateKey,
        eos_types::{
            EosNetwork,
            ProcessedTxIds,
        },
        eos_constants::{
            EOS_CHAIN_ID,
            EOS_NETWORK_KEY,
            EOS_ACCOUNT_NONCE,
            EOS_CHAIN_ID_DB_KEY,
            EOS_TOKEN_TICKER_KEY,
            PROCESSED_TX_IDS_KEY,
            EOS_ACCOUNT_NAME_KEY,
            EOS_PRIVATE_KEY_DB_KEY,
        },
        eos_utils::{
            convert_eos_network_to_bytes,
            convert_bytes_to_eos_network,
        },
    },
};

fn put_bytes_in_db(k: Bytes, v: Bytes) -> Result<()> { // TODO REINSTATE!
    Ok(())
}

// TODO pass in the db to all functions herein!
fn get_bytes_from_db(k: Bytes) -> Result<Bytes> { // TODO REINSTATE!
    Ok(vec![0u8])
}

pub fn get_eos_account_nonce_from_db<D>(
    db: &D
) -> Result<u64>
    where D: DatabaseInterface
{
    get_u64_from_db(db, &EOS_ACCOUNT_NONCE.to_vec())
}

pub fn put_eos_account_nonce_in_db<D>(
    db: &D,
    new_nonce: &u64,
) -> Result<()>
    where D: DatabaseInterface
{
    put_u64_in_db(db, &EOS_ACCOUNT_NONCE.to_vec(), new_nonce)
}

pub fn put_eos_token_ticker_in_db<D>(
    db: &D,
    name: &String,
) -> Result<()>
    where D: DatabaseInterface
{
    put_string_in_db(db, &EOS_TOKEN_TICKER_KEY.to_vec(), name)
}

pub fn get_eos_token_ticker_from_db<D>(
    db: &D,
) -> Result<String>
    where D: DatabaseInterface
{
    get_string_from_db(db, &EOS_TOKEN_TICKER_KEY.to_vec())
}

pub fn put_eos_account_name_in_db<D>(
    db: &D,
    name: &String,
) -> Result<()>
    where D: DatabaseInterface
{
    put_string_in_db(db, &EOS_ACCOUNT_NAME_KEY.to_vec(), name)
}

pub fn get_eos_account_name_from_db<D>(
    db: &D,
) -> Result<String>
    where D: DatabaseInterface
{
    get_string_from_db(db, &EOS_ACCOUNT_NAME_KEY.to_vec())
}

pub fn put_eos_chain_id_in_db<D>(
    db: &D,
    chain_id: &String
) -> Result<()>
    where D: DatabaseInterface
{
    put_string_in_db(db, &EOS_CHAIN_ID_DB_KEY.to_vec(), chain_id)
}

pub fn get_eos_chain_id_from_db<D>(
    db: &D,
) -> Result<String>
    where D: DatabaseInterface
{
    get_string_from_db(db, &EOS_CHAIN_ID_DB_KEY.to_vec())
}

pub fn get_processed_tx_ids_from_db<D>(
    db: &D,
) -> Result<ProcessedTxIds>
    where D: DatabaseInterface
{
    db.get(PROCESSED_TX_IDS_KEY.to_vec(), None)
        .and_then(|bytes| Ok(serde_json::from_slice(&bytes[..])?))
}

pub fn put_processed_tx_ids_in_db<D>(
    db: &D,
    processed_tx_ids: &ProcessedTxIds,
) -> Result<()>
    where D: DatabaseInterface
{
    db.put(
        PROCESSED_TX_IDS_KEY.to_vec(),
        serde_json::to_vec(processed_tx_ids)?,
        None,
    )
}

pub fn start_eos_db_transaction<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    state
        .db
        .start_transaction()
        .map(|_| {
            info!("✔ Database transaction begun for EOS block submission!");
            state
        })
}

pub fn end_eos_db_transaction<D>(
    state: EosState<D>,
) -> Result<EosState<D>>
    where D: DatabaseInterface
{
    state
        .db
        .end_transaction()
        .map(|_| {
            info!("✔ Database transaction ended for EOS block submission!");
            state
        })
}

pub fn put_eos_private_key_in_db<D>(
    db: &D,
    pk: &EosPrivateKey
) -> Result<()>
    where D: DatabaseInterface
{
    debug!("✔ Putting EOS private key into db...");
    db.put(EOS_PRIVATE_KEY_DB_KEY.to_vec(), pk.to_bytes(), Some(255))
    // FIXME This exposes the pk, do the trick from pBTC where we pass the db to a method on the struct!
}

pub fn get_eos_private_key_from_db<D>(
    db: &D
) -> Result<EosPrivateKey>
    where D: DatabaseInterface
{
    debug!("✔ Getting EOS private key from db...");
    db.get(EOS_PRIVATE_KEY_DB_KEY.to_vec(), Some(255))
        .and_then(|bytes| EosPrivateKey::from_slice(&bytes[..]))
}

pub fn get_eos_network_from_db() -> Result<EosNetwork> {
    debug!("✔ Getting EOS network from database...");
    get_bytes_from_db(EOS_NETWORK_KEY.to_vec())
        .and_then(|bytes| convert_bytes_to_eos_network(&bytes))
}

pub fn put_eos_network_in_db(network: &EosNetwork) -> Result<()> {
    debug!("✔ Adding EOS '{:?}' network to database...", network);
    put_bytes_in_db(
        EOS_NETWORK_KEY.to_vec(),
        convert_eos_network_to_bytes(network)?,
    )
}

#[cfg(test)]
mod tests {
    //use super::*; // TODO Tests!

}
