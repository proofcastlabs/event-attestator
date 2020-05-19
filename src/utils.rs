use tiny_keccak::keccak256;
use crate::{
    errors::AppError,
    types::{
        Bytes,
        Result,
    },
    constants::{
        U64_NUM_BYTES,
        DB_KEY_PREFIX,
    },
};

pub fn get_prefixed_db_key_hash(suffix: &str) -> [u8; 32] {
    keccak256(format!("{}{}",DB_KEY_PREFIX.to_string(), suffix).as_bytes())
}

pub fn convert_bytes_to_u64(bytes: &Bytes) -> Result<u64> {
    match bytes.len() {
        0..=7 => Err(AppError::Custom(
            "✘ Not enough bytes to convert to u64!"
                .to_string()
        )),
        U64_NUM_BYTES => {
            let mut arr = [0u8; U64_NUM_BYTES];
            let bytes = &bytes[..U64_NUM_BYTES];
            arr.copy_from_slice(bytes);
            Ok(u64::from_le_bytes(arr))
        }
        _ => Err(AppError::Custom(
            "✘ Too many bytes to convert to u64 without overflowing!"
                .to_string()
        )),
    }
}

fn left_pad_with_zero(string: &str) -> Result<String> {
    Ok(format!("0{}", string))
}

pub fn strip_hex_prefix(prefixed_hex : &str) -> Result<String> {
    let res = str::replace(prefixed_hex, "0x", "");
    match res.len() % 2 {
        0 => Ok(res),
        _ => left_pad_with_zero(&res),
    }
}


pub fn convert_u64_to_bytes(u_64: u64) -> Bytes {
    u_64.to_le_bytes().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_u64_to_bytes() {
        let u_64 = u64::max_value();
        let expected_result = [255,255,255,255,255,255,255,255];
        let result = convert_u64_to_bytes(u_64);
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_bytes_to_u64() {
        let bytes = vec![255,255,255,255,255,255,255,255];
        let expected_result = u64::max_value();
        let result = convert_bytes_to_u64(&bytes)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_not_strip_missing_hex_prefix_correctly() {
        let dummy_hex = "c0ffee";
        let expected_result = "c0ffee".to_string();
        let result = strip_hex_prefix(dummy_hex)
            .unwrap();
        assert!(result == expected_result)
    }

    #[test]
    fn should_left_pad_string_with_zero_correctly() {
        let dummy_hex = "0xc0ffee";
        let expected_result = "00xc0ffee".to_string();
        let result = left_pad_with_zero(dummy_hex)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_strip_hex_prefix_correctly() {
        let dummy_hex = "0xc0ffee";
        let expected_result = "c0ffee".to_string();
        let result = strip_hex_prefix(dummy_hex)
            .unwrap();
        assert!(result == expected_result)
    }
}
