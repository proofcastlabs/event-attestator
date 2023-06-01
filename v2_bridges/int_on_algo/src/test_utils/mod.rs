#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use common::{dictionaries::evm_algo::EvmAlgoTokenDictionaryEntry, errors::AppError, types::Result};
use common_algorand::AlgoSubmissionMaterial;
use common_eth::{convert_hex_to_eth_address, EthSubmissionMaterial};
use ethereum_types::Address as EthAddress;

macro_rules! write_int_paths_and_getter_fxn {
    ( $( $num:expr => $path:expr ),* ) => {
        paste! {
            $(const [<SAMPLE_INT_BLOCK_ $num>]: &str = $path;)*

            fn get_int_path_n(n: usize) -> Result<String> {
                match n {
                    $($num => Ok([<SAMPLE_INT_BLOCK_ $num>].to_string()),)*
                    _ => Err(AppError::Custom(format!("Cannot find sample INT block num: {}", n).into())),
                }
            }

            pub fn get_sample_int_submission_material_n(n: usize) -> EthSubmissionMaterial {
                EthSubmissionMaterial::from_str(&read_to_string(get_int_path_n(n).unwrap()).unwrap()).unwrap()
            }
        }
    }
}

macro_rules! write_algo_paths_and_getter_fxn {
    ( $( $num:expr => $path:expr ),* ) => {
        paste! {
            $(const [<SAMPLE_ALGO_BLOCK_ $num>]: &str = $path;)*

            fn get_algo_path_n(n: usize) -> Result<String> {
                match n {
                    $($num => Ok([<SAMPLE_ALGO_BLOCK_ $num>].to_string()),)*
                    _ => Err(AppError::Custom(format!("Cannot find sample ALGO block num: {}", n).into())),
                }
            }

            pub fn get_sample_algo_submission_material_n(n: usize) -> AlgoSubmissionMaterial {
                AlgoSubmissionMaterial::from_str(&read_to_string(get_algo_path_n(n).unwrap()).unwrap()).unwrap()
            }
        }
    }
}

write_int_paths_and_getter_fxn!(
    0 => "src/test_utils/int-block-12221813.json",
    1 => "src/test_utils/int-block-12221814.json",
    2 => "src/test_utils/int-block-12322610.json",
    3 => "src/test_utils/int-block-12322611.json",
    4 => "src/test_utils/int-block-12342413.json",
    5 => "src/test_utils/int-block-12342414.json"
);

write_algo_paths_and_getter_fxn!(
    0 => "src/test_utils/algo-block-20642396.json",
    1 => "src/test_utils/algo-block-20642397.json",
    2 => "src/test_utils/algo-block-20642398.json",
    3 => "src/test_utils/algo-block-21515430.json",
    4 => "src/test_utils/algo-block-21515431.json",
    5 => "src/test_utils/algo-block-21515432.json",
    6 => "src/test_utils/algo-block-21530957.json",
    7 => "src/test_utils/algo-block-21530958.json",
    8 => "src/test_utils/algo-block-21530959.json",
    9 => "src/test_utils/algo-block-17962555.json"
);

pub fn get_sample_vault_address() -> EthAddress {
    convert_hex_to_eth_address("0xE0806Ce04978224E27C6bB10E27fD30A7785ae9D").unwrap()
}

pub fn get_sample_router_address() -> EthAddress {
    convert_hex_to_eth_address("0xec1700a39972482d5db20e73bb3ffe6829b0c102").unwrap()
}

pub fn get_sample_contiguous_int_submission_json_strings_for_algo_address_peg_in() -> Vec<String> {
    vec![
        read_to_string(get_int_path_n(0).unwrap()).unwrap(),
        read_to_string(get_int_path_n(1).unwrap()).unwrap(),
    ]
}

pub fn get_sample_contiguous_int_submission_json_strings_for_app_id_peg_in() -> Vec<String> {
    vec![
        read_to_string(get_int_path_n(2).unwrap()).unwrap(),
        read_to_string(get_int_path_n(3).unwrap()).unwrap(),
    ]
}

pub fn get_sample_contiguous_int_submission_json_strings_for_msg_pack_encoded_user_data() -> Vec<String> {
    vec![
        read_to_string(get_int_path_n(4).unwrap()).unwrap(),
        read_to_string(get_int_path_n(5).unwrap()).unwrap(),
    ]
}

pub fn get_sample_contiguous_algo_submission_json_strings_for_asset_transfer_peg_out() -> Vec<String> {
    vec![
        read_to_string(get_algo_path_n(0).unwrap()).unwrap(),
        read_to_string(get_algo_path_n(1).unwrap()).unwrap(),
        read_to_string(get_algo_path_n(2).unwrap()).unwrap(),
    ]
}

pub fn get_sample_contiguous_algo_submission_json_strings_for_application_call_peg_out() -> Vec<String> {
    vec![
        read_to_string(get_algo_path_n(3).unwrap()).unwrap(),
        read_to_string(get_algo_path_n(4).unwrap()).unwrap(),
        read_to_string(get_algo_path_n(5).unwrap()).unwrap(),
    ]
}

pub fn get_sample_contiguous_algo_submission_json_strings_for_application_call_multi_peg_out() -> Vec<String> {
    vec![
        read_to_string(get_algo_path_n(6).unwrap()).unwrap(),
        read_to_string(get_algo_path_n(7).unwrap()).unwrap(),
        read_to_string(get_algo_path_n(8).unwrap()).unwrap(),
    ]
}

pub fn get_sample_evm_algo_dictionary_entry() -> EvmAlgoTokenDictionaryEntry {
    EvmAlgoTokenDictionaryEntry::from_str(
        "{\"algo_symbol\":\"ALGO\",\"evm_symbol\":\"EVM\",\"evm_decimals\": 18,\"algo_decimals\": 10,\"algo_asset_id\": 714666072,\"evm_address\": \"0x4262d1f878d191fbc66dca73bad57309916b1412\"}",
    ).unwrap()
}

mod tests {
    use super::*;

    #[test]
    fn submission_material_should_contain_algo_first_valid_round() {
        let material = get_sample_int_submission_material_n(0);
        assert!(material.algo_first_valid_round.is_some());
    }

    #[test]
    fn should_get_algo_submission_material_n() {
        get_sample_algo_submission_material_n(7);
    }
}
