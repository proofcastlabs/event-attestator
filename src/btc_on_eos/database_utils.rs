use crate::btc_on_eos::{
    errors::AppError,
    traits::DatabaseInterface,
    types::{
        Bytes,
        Result,
    },
};

fn put_bytes_in_db(k: Bytes, v: Bytes) -> Result<()> { // TODO Reinstate!
    Ok(())
}

fn get_bytes_from_db(k: Bytes) -> Result<Bytes> { // TODO Reinstate!
    Ok(vec![0u8])
}

pub fn put_u64_in_db<D>(
    db: &D,
    key: &Bytes,
    u_64: &u64,
) -> Result<()>
    where D: DatabaseInterface
{
    trace!("✔ Putting `u64` of {} in db...", u_64);
    db.put(key.to_vec(), u_64.to_le_bytes().to_vec(), None)
}

pub fn get_u64_from_db<D>(
    db: &D,
    key: &Bytes
) -> Result<u64>
    where D: DatabaseInterface
{
    trace!("✔ Getting `u64` from db...");
    db.get(key.to_vec(), None)
        .and_then(|bytes|
            match bytes.len() <= 8 {
                true => {
                    let mut array = [0; 8];
                    let bytes = &bytes[..array.len()];
                    array.copy_from_slice(bytes);
                    Ok(u64::from_le_bytes(array))
                },
                false => Err(AppError::Custom(
                    "✘ Too many bytes to convert to u64!".to_string()
                ))
            }
        )
}

pub fn put_usize_in_db(key: &Bytes, u_size: &usize) -> Result<()> {
    debug!("✔ Putting `usize` of {} in db...", u_size);
    put_bytes_in_db(key.to_vec(), u_size.to_le_bytes().to_vec())
}

pub fn get_usize_from_db(key: &Bytes) -> Result<usize> {
    debug!("✔ Getting `usize` from db...");
    get_bytes_from_db(key.to_vec())
        .and_then(|bytes|
            match bytes.len() <= 8 {
                true => {
                    let mut array = [0; 8];
                    let bytes = &bytes[..array.len()];
                    array.copy_from_slice(bytes);
                    Ok(usize::from_le_bytes(array))
                },
                false => Err(AppError::Custom(
                    "✘ Too many bytes to convert to usize!".to_string()
                ))
            }
        )
}

pub fn put_string_in_db(key: &Bytes, string: &String) -> Result<()> {
    debug!(
        "✔ Putting `string` of {} in db under key {}",
        string,
        hex::encode(key),
    );
    put_bytes_in_db(key.to_vec(), string.as_bytes().to_vec())
}

pub fn get_string_from_db(key: &Bytes) -> Result<String> {
    debug!("✔ Getting `string` from db under key: {}", hex::encode(key));
    get_bytes_from_db(key.to_vec())
        .map(|bytes|
            bytes
                .iter()
                .map(|byte| *byte as char)
                .collect::<String>()
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    /* TODO Reinstate!
    #[test]
    fn should_save_and_get_usize_from_db() {
        let key = vec![0xc0, 0xff, 0xee];
        let u_size = 1337;
        if let Err(e) = put_usize_in_db(&key, &u_size) {
            clear_test_database();
            panic!("Error saving eth account usize in db: {}", e);
        };
        match get_usize_from_db(&key) {
            Ok(usize_from_db) => {
                clear_test_database();
                assert!(usize_from_db == u_size);
            }
            Err(e) => {
                clear_test_database();
                panic!("Error getting usize from db: {}", e)
            }
        }
    }

    #[test]
    fn should_save_and_get_string_from_db() {
        let key = vec![0xc0, 0xff, 0xee];
        let string = "a string".to_string();
        if let Err(e) = put_string_in_db(&key, &string) {
            clear_test_database();
            panic!("Error saving string in db: {}", e);
        };
        match get_string_from_db(&key) {
            Ok(string_from_db) => {
                clear_test_database();
                assert!(string_from_db == string);
            }
            Err(e) => {
                clear_test_database();
                panic!("Error getting string from db: {}", e)
            }
        }
    }
    */
}
