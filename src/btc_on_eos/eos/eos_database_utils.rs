use crate::btc_on_eos::{
    types::Result,
    traits::DatabaseInterface,
    database_utils::{
        put_u64_in_db,
        get_u64_from_db,
        put_string_in_db,
        get_string_from_db,
    },
    eos::{
        eos_state::EosState,
        eos_types::ProcessedTxIds,
        eos_crypto::eos_private_key::EosPrivateKey,
        eos_constants::{
            EOS_ACCOUNT_NONCE,
            EOS_CHAIN_ID_DB_KEY,
            EOS_TOKEN_TICKER_KEY,
            PROCESSED_TX_IDS_KEY,
            EOS_ACCOUNT_NAME_KEY,
            EOS_PRIVATE_KEY_DB_KEY,
        },
    },
};

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
    pk.write_to_database(db, &EOS_PRIVATE_KEY_DB_KEY.to_vec())
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
