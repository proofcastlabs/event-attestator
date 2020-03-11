#![allow(dead_code)] //  TODO FIXME: Rm!
use eos_primitives::name_to_utf8;
use std::{
    str::from_utf8,
    convert::TryInto,
};
use crate::btc_on_eos::{
    errors::AppError,
    types::{
        Bytes,
        Result,
    },
    eos::{
        eos_types::EosRawTxData,
        eos_constants::EOS_NAME_BYTES_LEN,
    },
};

fn convert_bytes_to_u64(bytes: &Bytes) -> Result<u64> {
    match bytes.len() {
        EOS_NAME_BYTES_LEN => Ok(
            u64::from_le_bytes(
                bytes[..].try_into()?
            )
        ),
        _ => Err(AppError::Custom(
            "✘ Requires 8 bytes for u64 conversion!".to_string()
        ))
    }
}

fn convert_hex_string_to_u64(hex_string: &str) -> Result<u64> {
    convert_bytes_to_u64(&hex::decode(hex_string)?)
}

fn convert_u64_to_eos_utf8_string(num: u64) -> Result<String> {
    Ok(from_utf8(&name_to_utf8(num)[..12])?.to_string())
}

fn convert_bytes_to_eos_name(bytes: &Bytes) -> Result<String> {
    convert_bytes_to_u64(bytes)
        .and_then(convert_u64_to_eos_utf8_string)
}

fn convert_hex_string_to_eos_name(
    eos_hex_string: &str
) -> Result<String> {
    convert_hex_string_to_u64(eos_hex_string)
        .and_then(convert_u64_to_eos_utf8_string)
}

fn convert_bytes_to_utf8_string(bytes: &Bytes) -> Result<String> {
    Ok(from_utf8(bytes)?.to_string())
}

fn convert_hex_to_utf8_string(hex: &str) -> Result<String> {
    convert_bytes_to_utf8_string(&hex::decode(hex)?)
}

impl EosRawTxData {
    fn from_hex(hex: String) -> Result<Self> {
        let last_chunk_index = (hex.len().clone() / 2) - EOS_NAME_BYTES_LEN;
        let bytes = hex::decode(hex)?;
        Ok(
            EosRawTxData {
                sender: convert_bytes_to_eos_name(
                    &bytes[..EOS_NAME_BYTES_LEN]
                        .to_vec()
                )?,
                mint_nonce: convert_bytes_to_u64(
                    &bytes[last_chunk_index..]
                        .to_vec()
                )?,
                receiver: convert_bytes_to_eos_name(
                    &bytes[EOS_NAME_BYTES_LEN..EOS_NAME_BYTES_LEN * 2]
                        .to_vec()
                )?,
                eth_address: convert_bytes_to_utf8_string(
                    &bytes[EOS_NAME_BYTES_LEN * 4 + 1..last_chunk_index]
                        .to_vec()
                )?,
                asset_amount: convert_bytes_to_u64(
                    &bytes[EOS_NAME_BYTES_LEN * 2..EOS_NAME_BYTES_LEN * 3]
                        .to_vec()
                )?,
                asset_name: convert_bytes_to_utf8_string(
                    &bytes[EOS_NAME_BYTES_LEN * 3..EOS_NAME_BYTES_LEN * 4]
                        .to_vec()
                )?,
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_u64_to_eos_utf8_string() {
        let expected_result = "provabletokn".to_string();
        let num: u64 = 12531744380283593008;
        let result = convert_u64_to_eos_utf8_string(num)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_hex_string_to_u64() {
        let hex = "3021cd2a1eb3e9ad";
        let expected_result: u64 = 12531744380283593008;
        let result = convert_hex_string_to_u64(hex)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_bytes_to_u64() {
        let bytes = hex::decode("3021cd2a1eb3e9ad").unwrap();
        let expected_result: u64 = 12531744380283593008;
        let result = convert_bytes_to_u64(&bytes)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_eos_hex_string_to_eos_name() {
        let hex = "3021cd2a1eb3e9ad";
        let expected_result = "provabletokn".to_string();
        let result = convert_hex_string_to_eos_name(hex)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_bytes_to_eos_name() {
        let bytes = hex::decode("3021cd2a1eb3e9ad").unwrap();
        let expected_result = "provabletokn".to_string();
        let result = convert_bytes_to_eos_name(&bytes)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_hex_to_utf8_string() {
        let hex = "307830323664433641343335363144413841364137373535333862313932413365393336633046323942";
        let expected_result =
            "0x026dC6A43561DA8A6A775538b192A3e936c0F29B"
            .to_string();
        let result = convert_hex_to_utf8_string(hex)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_byte_to_utf8_string() {
        let bytes = hex::decode(
            "307830323664433641343335363144413841364137373535333862313932413365393336633046323942"
        ).unwrap();
        let expected_result =
            "0x026dC6A43561DA8A6A775538b192A3e936c0F29B"
            .to_string();
        let result = convert_bytes_to_utf8_string(&bytes)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_desrialize_eos_raw_tx_data_correctly() {
        let hex = "e0d2b86b1a3962343021cd2a1eb3e9ad672b00000000000004454f53000000002a3078303236644336413433353631444138413641373735353338623139324133653933366330463239426a01000000000000".to_string();
        let expected_asset_name = "EOS";
        let expected_mint_nonce: u64 = 362;
        let expected_asset_amount: u64 = 11111;
        let expected_sender = "all3manfr3di".to_string();
        let expected_receiver = "provabletokn".to_string();
        let expected_eth_address =
            "0x026dC6A43561DA8A6A775538b192A3e936c0F29B".to_string();
        let result = EosRawTxData::from_hex(hex)
            .unwrap();
        assert!(result.sender == expected_sender);
        assert!(result.receiver == expected_receiver);
        assert!(result.mint_nonce == expected_mint_nonce);
        assert!(result.eth_address == expected_eth_address);
        assert!(result.asset_amount == expected_asset_amount);
        assert!(result.asset_name.contains(expected_asset_name));
    }
}
