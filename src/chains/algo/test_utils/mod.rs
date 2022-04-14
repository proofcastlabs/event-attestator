#![cfg(test)]
use std::{fs::read_to_string, str::FromStr};

use paste::paste;
use rust_algorand::AlgorandBlock;

use crate::{errors::AppError, types::Result};

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

            pub fn get_all_sample_blocks() -> Vec<AlgorandBlock> {
                vec![
                    $(AlgorandBlock::from_str(&read_to_string($path).unwrap()).unwrap(),)*
                ]
            }
        }
    }
}

write_paths_and_getter_fxn!(
    0  => "src/chains/algo/test_utils/block-17962555.json",
    1  => "src/chains/algo/test_utils/block-17962556.json",
    2  => "src/chains/algo/test_utils/block-17962557.json",
    3  => "src/chains/algo/test_utils/block-17962558.json",
    4  => "src/chains/algo/test_utils/block-17962559.json",
    5  => "src/chains/algo/test_utils/block-17962560.json",
    6  => "src/chains/algo/test_utils/block-17962561.json",
    7  => "src/chains/algo/test_utils/block-17962562.json",
    8  => "src/chains/algo/test_utils/block-17962563.json",
    9  => "src/chains/algo/test_utils/block-17962564.json",
    10 => "src/chains/algo/test_utils/block-17962565.json"
);

pub fn get_sample_contiguous_blocks() -> Vec<AlgorandBlock> {
    let result = vec![0u8; 11];
    result.iter().enumerate().map(|(n, _)| get_sample_block_n(n)).collect()
}

pub fn get_sample_block_json_str_n(n: usize) -> String {
    read_to_string(get_path_n(n).unwrap()).unwrap()
}

pub fn get_sample_block_n(n: usize) -> AlgorandBlock {
    AlgorandBlock::from_str(&get_sample_block_json_str_n(n)).unwrap()
}

mod tests {
    use super::*;

    #[test]
    fn should_get_sample_block_n() {
        get_sample_block_n(0);
    }
}
