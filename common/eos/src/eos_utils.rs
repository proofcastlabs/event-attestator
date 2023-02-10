use common::{
    types::{Byte, Bytes, Result},
    utils::get_unix_timestamp_as_u32,
};
use eos_chain::Checksum256;

use crate::eos_constants::{EOS_MAX_EXPIRATION_SECS, EOS_SCHEDULE_DB_PREFIX};

pub fn convert_hex_to_checksum256<T: AsRef<[u8]>>(hex: T) -> Result<Checksum256> {
    convert_bytes_to_checksum256(&hex::decode(hex)?)
}

pub fn convert_bytes_to_checksum256(bytes: &[Byte]) -> Result<Checksum256> {
    match bytes.len() {
        32 => {
            let mut arr = [0; 32];
            arr.copy_from_slice(bytes);
            Ok(Checksum256::from(arr))
        },
        _ => Err(format!("✘ Wrong number of bytes. Expected 32, got {}", bytes.len()).into()),
    }
}

pub fn get_eos_schedule_db_key(version: u32) -> Bytes {
    format!("{}{}", EOS_SCHEDULE_DB_PREFIX, version).as_bytes().to_vec()
}

pub fn remove_symbol_from_eos_asset(eos_asset: &str) -> &str {
    eos_asset.split_whitespace().collect::<Vec<&str>>()[0]
}

pub fn get_symbol_from_eos_asset(eos_asset: &str) -> &str {
    eos_asset.split_whitespace().collect::<Vec<&str>>()[1]
}

pub fn get_eos_tx_expiration_timestamp_with_offset(offset: u32) -> Result<u32> {
    // NOTE: An EOS tx over the same params w/ the same timestamp results in the same
    // signature. This CAN happen organically such as a user pegging in the exact
    // same amount twice in a single block.
    get_unix_timestamp_as_u32().map(|timestamp| timestamp + EOS_MAX_EXPIRATION_SECS + offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_remove_symbol_from_eos_asset() {
        let amount = "1.23456789";
        let asset = format!("{} SAM", amount);
        let result = remove_symbol_from_eos_asset(&asset);
        assert_eq!(result, amount);
    }

    #[test]
    fn should_get_symbol_from_eos_asset() {
        let asset = "1.234 SAM";
        let result = get_symbol_from_eos_asset(asset);
        let expected_result = "SAM";
        assert_eq!(result, expected_result);
    }
}
