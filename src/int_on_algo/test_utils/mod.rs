#![cfg(test)]
use std::fs::read_to_string;

use crate::{chains::eth::eth_submission_material::EthSubmissionMaterial, errors::AppError, types::Result};

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
    0 => "src/int_on_algo/test_utils/int-block-1387181.json"
);

mod tests {
    use super::*;

    #[test]
    fn submission_material_should_contain_algo_first_valid_round() {
        let material = get_sample_int_submission_material_n(0);
        assert!(material.algo_first_valid_round.is_some());
    }
}
