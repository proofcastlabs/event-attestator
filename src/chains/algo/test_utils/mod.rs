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
        }
    }
}

write_paths_and_getter_fxn!(
    0 => "src/chains/algo/test_utils/block-17962555.json",
    1 => "src/chains/algo/test_utils/block-17962556.json"
);

fn get_sample_block_json_str_n(n: usize) -> String {
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
