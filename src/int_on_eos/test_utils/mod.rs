#![cfg(test)]
use std::fs::read_to_string;

use ethereum_types::Address as EthAddress;

use crate::{
    chains::{
        eos::eos_crypto::eos_private_key::EosPrivateKey,
        eth::{eth_crypto::eth_private_key::EthPrivateKey, eth_utils::convert_hex_to_eth_address},
    },
    dictionaries::eos_eth::{EosEthTokenDictionary, EosEthTokenDictionaryEntry, EosEthTokenDictionaryEntryJson},
    errors::AppError,
    types::Result,
};

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
        }
    }
}

macro_rules! write_eos_paths_and_getter_fxn {
    ( $( $num:expr => $path:expr ),* ) => {
        paste! {
            $(const [<SAMPLE_EOS_BLOCK_ $num>]: &str = $path;)*

            fn get_eos_path_n(n: usize) -> Result<String> {
                match n {
                    $($num => Ok([<SAMPLE_EOS_BLOCK_ $num>].to_string()),)*
                    _ => Err(AppError::Custom(format!("Cannot find sample EOS block num: {}", n).into())),
                }
            }
        }
    }
}

write_int_paths_and_getter_fxn!(
    0 => "src/int_on_eos/test_utils/int-block-12236005.json",
    1 => "src/int_on_eos/test_utils/int-block-12236006.json"
);

write_eos_paths_and_getter_fxn!(
    0 => "src/int_on_eos/test_utils/eos-init-block-213498746.json",
    1 => "src/int_on_eos/test_utils/eos-block-213499122.json"
);

pub fn get_sample_eos_init_block() -> String {
    read_to_string(get_eos_path_n(0).unwrap()).unwrap()
}

pub fn get_contiguous_int_block_json_strs() -> Vec<String> {
    vec![
        read_to_string(get_int_path_n(0).unwrap()).unwrap(),
        read_to_string(get_int_path_n(1).unwrap()).unwrap(),
    ]
}

pub fn get_sample_vault_address() -> EthAddress {
    convert_hex_to_eth_address("0xE0806Ce04978224E27C6bB10E27fD30A7785ae9D").unwrap()
}

pub fn get_sample_router_address() -> EthAddress {
    convert_hex_to_eth_address("0xec1700a39972482d5db20e73bb3ffe6829b0c102").unwrap()
}

pub fn get_sample_dictionary() -> EosEthTokenDictionary {
    EosEthTokenDictionary::new(vec![EosEthTokenDictionaryEntry::from_json(
        &EosEthTokenDictionaryEntryJson {
            eth_token_decimals: 18,
            eos_token_decimals: 8,
            eth_symbol: "TOK".to_string(),
            eos_symbol: "IOE".to_string(),
            eth_address: "0x4262d1f878d191fbc66dca73bad57309916b1412".to_string(),
            eos_address: "intoneostest".to_string(),
            eth_fee_basis_points: None,
            eos_fee_basis_points: None,
            accrued_fees: None,
            last_withdrawal: None,
        },
    )
    .unwrap()])
}

pub fn get_sample_eos_private_key() -> EosPrivateKey {
    EosPrivateKey::from_wallet_import_format("5KXpfu8A5E8zZcuKuigZFbVUQUPdwDedbKCEnjFdfpMdXgW318K").unwrap()
}

pub fn get_sample_int_private_key() -> EthPrivateKey {
    EthPrivateKey::from_slice(&hex::decode("e87a3a4b16ffc44c78d53f633157f0c08dc085a33483c2cbae78aa5892247e4c").unwrap())
        .unwrap()
}

pub fn get_sample_int_address() -> EthAddress {
    convert_hex_to_eth_address("0x49B9d619E3402de8867A8113C7bc204653F5DB4c").unwrap()
}

#[cfg(feature = "non-validating")]
pub fn get_sample_eos_submission_material_string() -> String {
    read_to_string(get_eos_path_n(1).unwrap()).unwrap()
}
