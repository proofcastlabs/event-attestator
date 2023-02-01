use crate::{
    traits::DatabaseInterface,
    types::{Byte, Result},
    utils::convert_bytes_to_u64,
};

pub fn put_string_in_db<D: DatabaseInterface>(db: &D, key: &[Byte], string: &str) -> Result<()> {
    debug!("✔ Putting `string` of {} in db under key {}", string, hex::encode(key));
    db.put(key.to_vec(), string.as_bytes().to_vec(), None)
}

pub fn get_string_from_db<D: DatabaseInterface>(db: &D, key: &[Byte]) -> Result<String> {
    debug!("✔ Getting `string` from db under key: {}", hex::encode(key));
    db.get(key.to_vec(), None)
        .map(|bytes| bytes.iter().map(|byte| *byte as char).collect::<String>())
}

pub fn put_u64_in_db<D: DatabaseInterface>(db: &D, key: &[Byte], u_64: u64) -> Result<()> {
    debug!("✔ Putting `u64` of {} in db...", u_64);
    db.put(key.to_vec(), u_64.to_le_bytes().to_vec(), None)
}

pub fn get_u64_from_db<D: DatabaseInterface>(db: &D, key: &[Byte]) -> Result<u64> {
    debug!("✔ Getting `u64` from db...");
    db.get(key.to_vec(), None)
        .and_then(|ref bytes| convert_bytes_to_u64(bytes))
}
