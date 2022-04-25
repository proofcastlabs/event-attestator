#![cfg(test)]
use std::fs::read_to_string;

use ethereum_types::Address as EthAddress;

use crate::{
    chains::eth::{eth_submission_material::EthSubmissionMaterial, eth_utils::convert_hex_to_eth_address},
    errors::AppError,
    types::Result,
};

macro_rules! write_paths_and_getter_fxn {
    ( $( $num:expr => $path:expr ),* ) => {
        paste! {
            $(const [<SAMPLE_BLOCK_ $num>]: &str = $path;)*

            fn get_path_n(n: usize) -> Result<String> {
                match n {
                    $($num => Ok([<SAMPLE_BLOCK_ $num>].to_string()),)*
                    _ => Err(AppError::Custom(format!("Cannot find sample block num: {}", n).into())),
                }
            }

            pub fn get_sample_int_submission_material_n(n: usize) -> EthSubmissionMaterial {
                EthSubmissionMaterial::from_str(&read_to_string(get_path_n(n).unwrap()).unwrap()).unwrap()
            }
        }
    }
}

write_paths_and_getter_fxn!(
    0 => "src/int_on_algo/test_utils/int-block-12208045.json",
    1 => "src/int_on_algo/test_utils/int-block-12208046.json",
    2 => "src/int_on_algo/test_utils/int-block-12208047.json",
    3 => "src/int_on_algo/test_utils/int-block-12208048.json",
    4 => "src/int_on_algo/test_utils/int-block-12208049.json"
);

pub fn get_sample_vault_address() -> EthAddress {
    convert_hex_to_eth_address("0xE0806Ce04978224E27C6bB10E27fD30A7785ae9D").unwrap()
}

pub fn get_sample_router_address() -> EthAddress {
    convert_hex_to_eth_address("0xec1700a39972482d5db20e73bb3ffe6829b0c102").unwrap()
}

pub fn get_sample_contiguous_int_submission_json_strings() -> Vec<String> {
    vec![
        read_to_string(get_path_n(0).unwrap()).unwrap(),
        read_to_string(get_path_n(1).unwrap()).unwrap(),
        read_to_string(get_path_n(2).unwrap()).unwrap(),
        read_to_string(get_path_n(3).unwrap()).unwrap(),
        read_to_string(get_path_n(4).unwrap()).unwrap(),
    ]
}
pub fn get_sample_contiguous_int_blocks() -> Vec<EthSubmissionMaterial> {
    get_sample_contiguous_int_submission_json_strings()
        .iter()
        .map(|s| EthSubmissionMaterial::from_str(s).unwrap())
        .collect()
}

mod tests {
    use super::*;

    #[test]
    fn submission_material_should_contain_algo_first_valid_round() {
        let material = get_sample_int_submission_material_n(0);
        assert!(material.algo_first_valid_round.is_some());
    }
}
