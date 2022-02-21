use std::fs::read_to_string;

use crate::{chains::btc::btc_submission_material::BtcSubmissionMaterial, errors::AppError, types::Result};

macro_rules! write_btc_paths_and_getter_fxn {
    ( $( $num:expr => $path:expr ),* ) => {
        paste! {
            $(const [<SAMPLE_BTC_BLOCK_ $num>]: &str = $path;)*
            fn get_btc_block_path_n(n: usize) -> Result<String> {
                match n {
                    $($num => Ok([<SAMPLE_BTC_BLOCK_ $num>].to_string()),)*
                    _ => Err(AppError::Custom(format!("Cannot find sample block num: {}", n).into())),
                }
            }

            pub fn get_sample_btc_submission_material_json_str_n(n: usize) -> String {
                read_to_string(&get_btc_block_path_n(n).unwrap()).unwrap()
            }
        }
    }
}

write_btc_paths_and_getter_fxn!(
    0 => "src/btc_on_int/test_utils/btc-testnet-block-2163202.json",
    1 => "src/btc_on_int/test_utils/btc-testnet-block-2163203.json",
    2 => "src/btc_on_int/test_utils/btc-testnet-block-2163204.json",
    3 => "src/btc_on_int/test_utils/btc-testnet-block-2163205.json"
);

macro_rules! write_eth_paths_and_getter_fxn {
    ( $( $num:expr => $path:expr ),* ) => {
        paste! {
            $(const [<SAMPLE_INT_BLOCK_ $num>]: &str = $path;)*
            fn get_eth_block_path_n(n: usize) -> Result<String> {
                match n {
                    $($num => Ok([<SAMPLE_INT_BLOCK_ $num>].to_string()),)*
                    _ => Err(AppError::Custom(format!("Cannot find sample block num: {}", n).into())),
                }
            }

            pub fn get_sample_eth_submission_material_json_str_n(n: usize) -> String {
                read_to_string(&get_eth_block_path_n(n).unwrap()).unwrap()
            }
        }
    }
}

write_eth_paths_and_getter_fxn!(
    0 => "src/btc_on_int/test_utils/eth-ropsten-block-11980265.json"
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_btc_block_str() {
        get_sample_btc_submission_material_json_str_n(0);
    }

    #[test]
    fn should_get_eth_block_str() {
        get_sample_eth_submission_material_json_str_n(0);
    }
}
