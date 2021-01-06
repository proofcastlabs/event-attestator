use crate::{
    chains::eos::eos_constants::EOS_NUM_DECIMALS,
    types::{NoneError, Result},
};

pub fn convert_eos_asset_to_u64(eos_asset: &str) -> Result<u64> {
    Ok(eos_asset
        .replace(".", "")
        .split_whitespace()
        .next()
        .ok_or(NoneError("Error converting EOS asset to u64!"))?
        .parse()?)
}

pub fn convert_u64_to_eos_asset(amount: u64) -> String {
    let token_symbol = "EOS";
    let mut amount_string = amount.to_string();
    let asset = match amount_string.len() {
        0 => "0.0000".to_string(),
        1 => format!("0.000{}", amount_string),
        2 => format!("0.00{}", amount_string),
        3 => format!("0.0{}", amount_string),
        4 => format!("0.{}", amount_string),
        _ => {
            amount_string.insert(amount_string.len() - EOS_NUM_DECIMALS, '.');
            amount_string
        },
    };
    format!("{} {}", asset, token_symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_eos_asset_to_u64() {
        let expected_results = vec![
            123456789123456789 as u64,
            12345678912345678 as u64,
            1234567891234567 as u64,
            123456789123456 as u64,
            12345678912345 as u64,
            1234567891234 as u64,
            123456789123 as u64,
            12345678912 as u64,
            1234567891 as u64,
            123456789 as u64,
            12345678 as u64,
            1234567 as u64,
            123456 as u64,
            12345 as u64,
            1234 as u64,
            123 as u64,
            12 as u64,
            1 as u64,
            0 as u64,
        ];
        vec![
            "123456789.123456789 SAM",
            "12345678.912345678 SAM",
            "1234567.891234567 SAM",
            "123456.789123456 SAM",
            "12345.678912345 SAM",
            "1234.567891234 SAM",
            "123.456789123 SAM",
            "12.345678912 SAM",
            "1.234567891 SAM",
            "0.123456789 SAM",
            "0.012345678 SAM",
            "0.001234567 SAM",
            "0.000123456 SAM",
            "0.000012345 SAM",
            "0.000001234 SAM",
            "0.000000123 SAM",
            "0.000000012 SAM",
            "0.000000001 SAM",
            "0.000000000 SAM",
        ]
        .iter()
        .map(|eos_asset| convert_eos_asset_to_u64(eos_asset).unwrap())
        .zip(expected_results.iter())
        .for_each(|(result, expected_result)| assert_eq!(&result, expected_result));
    }

    #[test]
    fn should_convert_u64_to_eos_asset() {
        let expected_results = vec![
            "12345678912345.6789 EOS",
            "1234567891234.5678 EOS",
            "123456789123.4567 EOS",
            "12345678912.3456 EOS",
            "1234567891.2345 EOS",
            "123456789.1234 EOS",
            "12345678.9123 EOS",
            "1234567.8912 EOS",
            "123456.7891 EOS",
            "12345.6789 EOS",
            "1234.5678 EOS",
            "123.4567 EOS",
            "12.3456 EOS",
            "1.2345 EOS",
            "0.1234 EOS",
            "0.0123 EOS",
            "0.0012 EOS",
            "0.0001 EOS",
            "0.0000 EOS",
        ];
        vec![
            123456789123456789 as u64,
            12345678912345678 as u64,
            1234567891234567 as u64,
            123456789123456 as u64,
            12345678912345 as u64,
            1234567891234 as u64,
            123456789123 as u64,
            12345678912 as u64,
            1234567891 as u64,
            123456789 as u64,
            12345678 as u64,
            1234567 as u64,
            123456 as u64,
            12345 as u64,
            1234 as u64,
            123 as u64,
            12 as u64,
            1 as u64,
            0 as u64,
        ]
        .iter()
        .map(|u_64| convert_u64_to_eos_asset(*u_64))
        .zip(expected_results.iter())
        .for_each(|(result, expected_result)| assert_eq!(&result, expected_result));
    }
}
