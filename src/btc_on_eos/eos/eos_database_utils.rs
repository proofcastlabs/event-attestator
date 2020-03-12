use crate::btc_on_eos::{
    traits::DatabaseInterface,
    types::{
        Bytes,
        Result,
    },
    database_utils::{
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
            EOS_NETWORK_KEY,
            EOS_CHAIN_ID_DB_KEY,
            PROCESSED_TX_IDS_KEY,
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

pub fn put_eos_chain_id_in_db(chain_id: &String) -> Result<()> {
    debug!("✔ Putting EOS chain ID of '{}' into db...", chain_id);
    put_string_in_db(&EOS_CHAIN_ID_DB_KEY.to_vec(), chain_id)
}

pub fn get_eos_chain_id_from_db() -> Result<String> {
    debug!("✔ Getting EOS chain ID from db...");
    get_string_from_db(&EOS_CHAIN_ID_DB_KEY.to_vec())
}

pub fn put_eos_private_key_in_db(pk: &EosPrivateKey) -> Result<()> {
    debug!("✔ Saving EOS private key into db...");
    put_bytes_in_db(EOS_PRIVATE_KEY_DB_KEY.to_vec(), pk.to_bytes())
}

pub fn get_eos_private_key_from_db() -> Result<EosPrivateKey> {
    debug!("✔ Getting EOS private key from db...");
    get_bytes_from_db(EOS_PRIVATE_KEY_DB_KEY.to_vec())
        .and_then(|bytes|
            EosPrivateKey::from_slice(&bytes[..])
        )
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
