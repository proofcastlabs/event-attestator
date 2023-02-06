fn get_x_num_zeroes_string(num_zeroes: usize) -> String {
    let zeroes = vec![0u8; num_zeroes];
    zeroes.iter().fold(String::new(), |acc, e| format!("{acc}{e}"))
}

pub(super) fn convert_u64_to_x_decimal_eos_asset(value: u64, num_decimals: usize, token_symbol: &str) -> String {
    let mut amount_string = value.to_string();
    let amount_string_length = amount_string.len();
    let asset = if amount_string_length == 0 {
        format!("0.{}", get_x_num_zeroes_string(num_decimals))
    } else if amount_string_length < num_decimals {
        format!(
            "0.{}{amount_string}",
            get_x_num_zeroes_string(num_decimals - amount_string_length)
        )
    } else if amount_string_length == num_decimals {
        format!("0.{amount_string}")
    } else {
        amount_string.insert(amount_string.len() - num_decimals, '.');
        amount_string
    };
    format!("{} {}", asset, token_symbol)
}

#[cfg(test)]
mod tests {
    use common::chains::btc::btc_constants::BTC_NUM_DECIMALS;

    use super::*;

    #[test]
    fn should_convert_u64_to_8_decimal_eos_asset() {
        let symbol = "SAM";
        let expected_results = vec![
            "1234567891.23456789 SAM",
            "123456789.12345678 SAM",
            "12345678.91234567 SAM",
            "1234567.89123456 SAM",
            "123456.78912345 SAM",
            "12345.67891234 SAM",
            "1234.56789123 SAM",
            "123.45678912 SAM",
            "12.34567891 SAM",
            "1.23456789 SAM",
            "0.12345678 SAM",
            "0.01234567 SAM",
            "0.00123456 SAM",
            "0.00012345 SAM",
            "0.00001234 SAM",
            "0.00000123 SAM",
            "0.00000012 SAM",
            "0.00000001 SAM",
            "0.00000000 SAM",
        ];
        vec![
            123456789123456789_u64,
            12345678912345678_u64,
            1234567891234567_u64,
            123456789123456_u64,
            12345678912345_u64,
            1234567891234_u64,
            123456789123_u64,
            12345678912_u64,
            1234567891_u64,
            123456789_u64,
            12345678_u64,
            1234567_u64,
            123456_u64,
            12345_u64,
            1234_u64,
            123_u64,
            12_u64,
            1_u64,
            0_u64,
        ]
        .iter()
        .map(|u_64| convert_u64_to_x_decimal_eos_asset(*u_64, BTC_NUM_DECIMALS, symbol))
        .zip(expected_results.iter())
        .for_each(|(result, expected_result)| assert_eq!(&result, expected_result));
    }

    #[test]
    fn should_get_x_num_zeroes_string() {
        let vec = vec![0u8; 10];
        let results = vec
            .iter()
            .enumerate()
            .map(|(i, _)| get_x_num_zeroes_string(i))
            .collect::<Vec<String>>();
        let expected_results = vec![
            "",
            "0",
            "00",
            "000",
            "0000",
            "00000",
            "000000",
            "0000000",
            "00000000",
            "000000000",
        ];
        results
            .iter()
            .enumerate()
            .for_each(|(i, result)| assert_eq!(result, expected_results[i]));
    }
}
