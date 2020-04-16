use crate::{
    errors::AppError,
    constants::U64_NUM_BYTES,
    types::{
        Bytes,
        Result,
    },
};

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
}
