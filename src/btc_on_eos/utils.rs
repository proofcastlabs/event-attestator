use crate::{
    types::{Byte, Bytes, Result},
    btc_on_eos::constants::{
        U64_NUM_BYTES,
        BTC_NUM_DECIMALS,
    },
};

pub fn convert_eos_asset_to_u64(eos_asset: &str) -> Result<u64> { //TODO test!
    Ok(
        eos_asset
            .replace(".", "")
            .split_whitespace()
            .collect::<Vec<&str>>()[0]
            .parse::<u64>()?
    )
}

pub fn convert_u64_to_eos_asset( // TODO Test
    value: u64,
    token_symbol: &str,
) -> String {
    let mut amount_string = format!("{}", value);
    let asset = match amount_string.len() {
        0 => "0.00000000".to_string(),
        1 => format!("0.0000000{}", amount_string),
        2 => format!("0.000000{}", amount_string),
        3 => format!("0.00000{}", amount_string),
        4 => format!("0.0000{}", amount_string),
        5 => format!("0.000{}", amount_string),
        6 => format!("0.00{}", amount_string),
        7 => format!("0.0{}", amount_string),
        8 => format!("0.{}", amount_string),
        _ => {
            amount_string.insert(
                amount_string.len() - BTC_NUM_DECIMALS,
                '.'
            );
            amount_string
        }
    };
    format!("{} {}", asset, token_symbol)
}

pub fn convert_bytes_to_u64(bytes: &[Byte]) -> Result<u64> {
    match bytes.len() {
        0..=7 => Err("✘ Not enough bytes to convert to u64!".into()),
        U64_NUM_BYTES => {
            let mut arr = [0u8; U64_NUM_BYTES];
            let bytes = &bytes[..U64_NUM_BYTES];
            arr.copy_from_slice(bytes);
            Ok(u64::from_le_bytes(arr))
        }
        _ => Err("✘ Too many bytes to convert to u64 without overflowing!".into()),
    }
}

pub fn convert_u64_to_bytes(u_64: u64) -> Bytes {
    u_64.to_le_bytes().to_vec()
}
