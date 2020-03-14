use serde_json::Value;
use eos_primitives::Checksum256;
use ethereum_types::{
    U256,
    H256,
    Address as EthAddress
};
use crate::btc_on_eos::{
    errors::AppError,
    constants::EOS_TOKEN_TICKER,
    types::{
        Bytes,
        Result,
    },
    constants::{
        HASH_LENGTH,
        U64_NUM_BYTES,
    },
    eos::{
        eos_types::EosAmount,
        eos_constants::EOS_NUM_DECIMALS,
    },
};

pub fn convert_eos_asset_to_u64(eos_asset: String) -> Result<u64> {
    Ok(
        eos_asset
            .split_whitespace()
            .collect::<Vec<&str>>()[0]
            .parse::<u64>()?
    )
}

pub fn convert_u64_to_eos_asset(u_64: u64) -> String {
    format!("{} {}", u_64, EOS_TOKEN_TICKER)
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

pub fn convert_u64_to_bytes(u_64: &u64) -> Bytes {
    u_64.to_le_bytes().to_vec()
}

pub fn convert_hex_to_checksum256(hex: &String) -> Result<Checksum256> { //TODO: Test!
    let mut arr = [0; 32];
    let bytes = hex::decode(hex)?;
    arr.copy_from_slice(&bytes);
    Ok(Checksum256::from(arr))
}

pub fn convert_u256_to_32_byte_wide_zero_padded_hex(
    u256: U256
) -> String {
    format!("{:0>64}", format!("{:x}", u256))
}

pub fn convert_eth_address_to_32_byte_wide_zero_padded_hex(
    eth_address: EthAddress
) -> String {
    format!("{:0>64}", format!("{:x}", eth_address))
}

pub fn strip_new_line_chars(string: String) -> String {
    string.replace("\n", "")
}

pub fn convert_u256_to_eos_amount(amount: U256) -> Result<EosAmount> {
    let mut amount_string = amount.to_string();
    match amount_string.len() {
        0 => Err(AppError::Custom(
            "✘ Cannot convert zero ETH amount to EOS amount!".to_string()
        )),
        1 => Ok(format!("0.000{} EOS", amount_string)),
        2 => Ok(format!("0.00{} EOS", amount_string)),
        3 => Ok(format!("0.0{} EOS", amount_string)),
        4 => Ok(format!("0.{} EOS", amount_string)),
        _ => {
            amount_string.insert(
                amount_string.len() - EOS_NUM_DECIMALS,
                '.'
            );
            Ok(format!("{} EOS", amount_string))
        }
    }
}

pub fn convert_eos_amount_to_u256(eos_amount: String) -> Result<U256> {
    convert_dec_str_to_u256(
        &eos_amount
            .replace(" EOS", "")
            .replace(".", "")
    )
}

pub fn convert_dec_str_to_u256(dec_str: &str) -> Result<U256> {
    match U256::from_dec_str(dec_str) {
        Ok(u256) => Ok(u256),
        Err(e) => Err(AppError::Custom(
            format!("✘ Error converting decimal string to u256:\n{:?}", e)
        ))
    }
}

pub fn convert_h256_to_prefixed_hex(hash: H256) -> Result <String> {
    Ok(format!("0x{}", hex::encode(hash)))
}

pub fn convert_h256_to_bytes(hash: H256) -> Bytes {
    hash.as_bytes().to_vec()
}

pub fn convert_bytes_to_h256(bytes: &Bytes) -> Result<H256> {
    match bytes.len() {
        32 => Ok(H256::from_slice(&bytes[..])),
        _ => Err(AppError::Custom(
            "✘ Not enough bytes to convert to h256!".to_string()
        ))
    }
}

pub fn convert_usize_to_bytes(num: &usize) -> Bytes {
    num.to_le_bytes().to_vec()
}

pub fn convert_bytes_to_usize(bytes: &Bytes) -> Result<usize> {
    match bytes.len() >= 8 {
        true => {
            let mut array = [0; 8];
            let bytes = &bytes[..array.len()];
            array.copy_from_slice(bytes);
            Ok(usize::from_le_bytes(array))
        },
        false => Err(AppError::Custom(
            "✘ Not enough bytes to convert to usize!".to_string()
        ))
    }
}

pub fn convert_json_value_to_string(value: Value) -> Result<String> { // TODO: Test!
    Ok(value.as_str()?.to_string())
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

pub fn decode_hex(hex_to_decode: String) -> Result<Vec<u8>> {
    Ok(hex::decode(hex_to_decode)?)
}

pub fn decode_prefixed_hex(hex_to_decode: String) -> Result<Vec<u8>> {
    strip_hex_prefix(&hex_to_decode)
        .and_then(decode_hex)
}

pub fn get_not_in_state_err(substring: &str) -> String {
    format!("✘ No {} in state!" , substring)
}

pub fn get_no_overwrite_state_err(substring: &str) -> String {
    format!("✘ Cannot overwrite {} in state!" , substring)
}

pub fn convert_hex_to_u256(hex: String) -> Result<U256> {
    Ok(U256::from(&decode_prefixed_hex(hex)?[..]))
}

pub fn convert_hex_to_bytes(hex: String) -> Result<Bytes> {
    Ok(hex::decode(strip_hex_prefix(&hex.to_string())?)?)
}

pub fn check_hex_is_valid_ethereum_address(hex: &String) -> bool {
    match decode_hex(str::replace(hex, "0x", "")) {
        Err(_) => false,
        Ok(decoded_hex) => match decoded_hex.len() {
            20 => true,
            _ => false,
        }
    }
}

pub fn convert_hex_to_address(hex: String) -> Result<EthAddress> {
    Ok(EthAddress::from_slice(&decode_prefixed_hex(hex)?))
}

pub fn convert_hex_to_h256(hex: String) -> Result<H256> {
    decode_prefixed_hex(hex)
        .and_then(|bytes| match bytes.len() {
            HASH_LENGTH => Ok(H256::from_slice(&bytes)),
            0..HASH_LENGTH => Err(
                AppError::Custom(
                    format!("✘ Too few bytes in hex to create H256 type!")
                )
            ),
            _ => Err(
                AppError::Custom(
                    format!("✘ Too many bytes in hex to create H256 type!")
                )
            )
        })
}

pub fn convert_hex_strings_to_h256s(hex_strings: Vec<String>) -> Result<Vec<H256>> {
    let hashes: Result<Vec<H256>> = hex_strings
        .into_iter()
        .map(|hex_string| convert_hex_to_h256(hex_string.to_string()))
        .collect();
    Ok(hashes?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use crate::btc_on_eos::{
        constants::{
            HASH_HEX_CHARS,
            HEX_PREFIX_LENGTH,
        },
    };

    fn get_sample_block_hash() -> &'static str {
        "0x1ddd540f36ea0ed23e732c1709a46c31ba047b98f1d99e623f1644154311fe10"
    }

    fn get_sample_h256() -> H256 {
        convert_hex_to_h256(get_sample_block_hash().to_string())
            .unwrap()
    }

    #[test]
    fn should_decode_none_prefixed_hex_correctly() {
        let none_prefixed_hex = "c0ffee";
        assert!(!none_prefixed_hex.contains("x"));
        let expected_result = [192, 255, 238];
        let result = decode_hex(none_prefixed_hex.to_string())
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

    #[test]
    fn should_not_strip_missing_hex_prefix_correctly() {
        let dummy_hex = "c0ffee";
        let expected_result = "c0ffee".to_string();
        let result = strip_hex_prefix(dummy_hex)
            .unwrap();
        assert!(result == expected_result)
    }

    #[test]
    fn should_convert_hex_to_address_correcty() {
        let address_hex = "0xb2930b35844a230f00e51431acae96fe543a0347";
        let result = convert_hex_to_address(address_hex.to_string())
            .unwrap();
        let expected_result = decode_prefixed_hex(address_hex.to_string())
            .unwrap();
        let expected_result_bytes = &expected_result[..];
        assert!(result.as_bytes() == expected_result_bytes);
    }

    #[test]
    fn should_convert_unprefixed_hex_to_bytes_correctly() {
        let hex = "c0ffee".to_string();
        let expected_result = [ 192, 255, 238 ];
        let result = convert_hex_to_bytes(hex).unwrap();
        assert!(result == expected_result)
    }

    #[test]
    fn should_convert_prefixed_hex_to_bytes_correctly() {
        let hex = "0xc0ffee".to_string();
        let expected_result = [ 192, 255, 238 ];
        let result = convert_hex_to_bytes(hex).unwrap();
        assert!(result == expected_result)
    }

    #[test]
    fn should_get_no_state_err_string() {
        let thing = "thing".to_string();
        let expected_result = "✘ No thing in state!";
        let result = get_not_in_state_err(&thing);
        assert!(result == expected_result)
    }

    #[test]
    fn should_get_no_overwrite_err_string() {
        let thing = "thing".to_string();
        let expected_result = "✘ Cannot overwrite thing in state!";
        let result = get_no_overwrite_state_err(&thing);
        assert!(result == expected_result)
    }

    #[test]
    fn should_convert_hex_to_u256_correctly() {
        let hex = "0xc0ffee";
        let expected_result: u128 = 12648430;
        let result = convert_hex_to_u256(hex.to_string())
            .unwrap();
        assert!(result.as_u128() == expected_result)
    }

    #[test]
    fn should_decode_prefixed_hex_correctly() {
        let prefixed_hex = "0xc0ffee";
        let mut chars = prefixed_hex.chars();
        assert!("0" == chars.next().unwrap().to_string());
        assert!("x" == chars.next().unwrap().to_string());
        let expected_result = [192, 255, 238];
        let result = decode_prefixed_hex(prefixed_hex.to_string())
            .unwrap();
        assert!(result == expected_result)
    }
        #[test]
    fn should_convert_hex_to_h256_correctly() {
        let dummy_hash = "0xc5acf860fa849b72fc78855dcbc4e9b968a8af5cdaf79f03beeca78e6a9cec8b";
        assert!(dummy_hash.len() == HASH_HEX_CHARS + HEX_PREFIX_LENGTH);
        let result = convert_hex_to_h256(dummy_hash.to_string())
            .unwrap();
        let expected_result = decode_prefixed_hex(dummy_hash.to_string())
            .unwrap();
        let expected_result_bytes = &expected_result[..];
        assert!(result.as_bytes() == expected_result_bytes);
    }

    #[test]
    fn should_fail_to_convert_short_hex_to_h256_correctly() {
        let short_hash = "0xc5acf860fa849b72fc78855dcbc4e9b968a8af5cdaf79f03beeca78e6a9cec";
        assert!(short_hash.len() < HASH_HEX_CHARS + HEX_PREFIX_LENGTH);
        match convert_hex_to_h256(short_hash.to_string()) {
            Err(AppError::Custom(e)) => assert!(e == "✘ Too few bytes in hex to create H256 type!"),
            _ => panic!("Should have errored ∵ of short hash!")
        }
    }

    #[test]
    fn should_fail_to_convert_long_hex_to_h256_correctly() {
        let long_hash = "0xc5acf860fa849b72fc78855dcbc4e9b968a8af5cdaf79f03beeca78e6a9cecffff";
        assert!(long_hash.len() > HASH_HEX_CHARS + HEX_PREFIX_LENGTH);
        match convert_hex_to_h256(long_hash.to_string()) {
            Err(AppError::Custom(e)) => assert!(
                e == "✘ Too many bytes in hex to create H256 type!"
            ),
            _ => panic!("Should have errored ∵ of short hash!")
        }
    }

    #[test]
    fn should_fail_to_convert_invalid_hex_to_h256_correctly() {
        let long_hash = "0xc5acf860fa849b72fc78855dcbc4e9b968a8af5cdaf79f03beeca78e6a9cecffzz";
        assert!(long_hash.len() > HASH_HEX_CHARS + HEX_PREFIX_LENGTH);
        assert!(long_hash.contains("z"));
        match convert_hex_to_h256(long_hash.to_string()) {
            Err(AppError::HexError(e)) => assert!(
                e.to_string().contains("Invalid")
            ),
            Err(AppError::Custom(_)) => panic!("Should be hex error!"),
            _ => panic!("Should have errored ∵ of invalid hash!")
        }
    }

    #[test]
    fn should_convert_hex_strings_to_h256s() {
        let str1 = "0xebfa2e7610ea186fa3fa97bbaa5db80cce033dfff7e546c6ee05493dbcbfda7a".to_string();
        let str2 = "0x08075826de57b85238fe1728a37b366ab755b95c65c59faec7b0f1054fca1654".to_string();
        let expected_result1 = convert_hex_to_h256(str1.clone()).unwrap();
        let expected_result2 = convert_hex_to_h256(str2.clone()).unwrap();
        let hex_strings: Vec<String> = vec!(str1, str2);
        let results: Vec<H256> = convert_hex_strings_to_h256s(hex_strings)
            .unwrap();
        assert!(results[0] == expected_result1);
        assert!(results[1] == expected_result2);
    }

    #[test]
    fn should_convert_usize_to_bytes() {
        let num = 1337;
        let expected_result = vec![57, 5, 0, 0, 0, 0, 0, 0];
        let result = convert_usize_to_bytes(&num);
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_bytes_to_usize() {
        let bytes = vec![57, 5, 0, 0, 0, 0, 0, 0];
        let expected_result = 1337;
        let result = convert_bytes_to_usize(&bytes)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_h256_to_bytes() {
        let hash = H256::zero();
        let expected_result = vec![
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0
        ];
        let result = convert_h256_to_bytes(hash);
        assert!(expected_result == result);
    }

    #[test]
    fn should_convert_bytes_to_h256() {
        let hex_string = "ebfa2e7610ea186fa3fa97bbaa5db80cce033dfff7e546c6ee05493dbcbfda7a"
            .to_string();
        let expected_result = convert_hex_to_h256(hex_string.clone())
            .unwrap();
        let bytes = hex::decode(hex_string)
            .unwrap();
        let result = convert_bytes_to_h256(&bytes)
            .unwrap();
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_decimal_string_to_u256() {
        let expected_result = 1337;
        let dec_str = "1337";
        let result = convert_dec_str_to_u256(dec_str)
            .unwrap();
        assert!(result.as_usize() == expected_result);
    }

    #[test]
    fn should_fail_to_convert_non_decimal_string_to_u256() {
        let expected_error = "✘ Error converting decimal string";
        let dec_str = "abcd";
        match convert_dec_str_to_u256(dec_str) {
            Err(AppError::Custom(e)) => assert!(e.contains(expected_error)),
            _ => panic!("Should not have converted non decimal string!")
        }
    }

    #[test]
    fn should_convert_h256_to_prefixed_hex_correctly() {
        let expected_result = get_sample_block_hash();
        let hash = get_sample_h256();
        let result = convert_h256_to_prefixed_hex(hash)
            .unwrap();
        assert!(result == expected_result);
    }

    macro_rules! convert_u256_to_eos {
        ($( $amount:expr => $expected_result:expr ),*) => {
            {
                $( // Note: Uses the repeats!
                    let u256 = U256::from_dec_str($amount)
                        .unwrap();
                    let result = convert_u256_to_eos_amount(u256)
                        .unwrap();
                    assert!(result == $expected_result);
                )*
            }
        };
    }

    #[test]
    fn should_convert_u256_to_eos_amount() {
        convert_u256_to_eos! {
            "1" => "0.0001 EOS",
            "12" => "0.0012 EOS",
            "123" => "0.0123 EOS",
            "1234" => "0.1234 EOS",
            "12345" => "1.2345 EOS",
            "123456" => "12.3456 EOS",
            "1234567" => "123.4567 EOS",
            "12345678" => "1234.5678 EOS",
            "123456789" => "12345.6789 EOS"
        }
    }

    macro_rules! convert_eos_amount_to_u256 {
        ($( $amount:expr => $expected_result:expr ),*) => {
            {
                $( // Note: Uses the repeats!
                    let result = convert_eos_amount_to_u256($amount.to_string())
                        .unwrap();
                    let expected_result = U256::from_dec_str($expected_result)
                        .unwrap();
                    assert!(result == expected_result);
                )*
            }
        };
    }

    #[test]
    fn should_convert_eos_amount_to_u256() {
        convert_eos_amount_to_u256! {
            "0.0001 EOS" => "1",
            "0.0012 EOS" => "12",
            "0.0123 EOS" => "123",
            "0.1234 EOS" => "1234",
            "1.2345 EOS" => "12345",
            "12.3456 EOS" => "123456",
            "123.4567 EOS" => "1234567",
            "1234.5678 EOS" => "12345678",
            "12345.6789 EOS" => "123456789"
        }
    }

    #[test]
    fn valid_eth_address_should_be_valid() {
        assert!(check_hex_is_valid_ethereum_address(
            &"0xd6f026989ec8f928edcf4edc250aaad3dd14cdae".to_string()
        ))
    }

    #[test]
    fn invalid_eth_addresses_should_be_invalid() {
        assert!(!check_hex_is_valid_ethereum_address(
            &"0xd6f026989ec8f928edcf4edc250aaad3dd14cda".to_string()
        ));
        assert!(!check_hex_is_valid_ethereum_address(
            &"0xd6f026989ec8f928edcf4edc250aaad3dd14cdaee".to_string()
        ));
        assert!(!check_hex_is_valid_ethereum_address(
            &"0xd6f026989ec8f928edcf4edc250aaad3dd14cdaez".to_string()
        ));
    }

    #[test]
    fn should_strip_newline_chars() {
        let new_line_char = "\n";
        let string = "a string".to_string();
        let test_vector = format!("{}{}", string, new_line_char);
        let length_before = test_vector.len();
        assert!(test_vector.contains(new_line_char));
        let result = strip_new_line_chars(string);
        let length_after = result.len();
        assert!(length_after < length_before);
        assert!(length_after == length_before - new_line_char.len());
    }

    #[test]
    fn should_convert_u256_to_padded_hex() {
        let u256 = U256::from_dec_str("12312321").unwrap();
        let expected_result = "0000000000000000000000000000000000000000000000000000000000bbdf01"
            .to_string();
        let result = convert_u256_to_32_byte_wide_zero_padded_hex(u256);
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_eth_address_to_padded_hex() {
        let eth_address = EthAddress::from_str(
            "1739624f5cd969885a224da84418d12b8570d61a"
        ).unwrap();
        let expected_result = "0000000000000000000000001739624f5cd969885a224da84418d12b8570d61a"
            .to_string();
        let result = convert_eth_address_to_32_byte_wide_zero_padded_hex(
            eth_address
        );
        assert!(result == expected_result);
    }

    #[test]
    fn should_convert_hex_to_checksum256() {
        let hex =
            "4f72e85ee91bb26bf223f0ad1e08e8ac11a143b4eb1ac9854e4e726e85cc9b51"
                .to_string();
        if let Err(e) = convert_hex_to_checksum256(&hex) {
            panic!("Error converting hex to checksum! {}", e);
        }
    }
}
