use crate::{chains::eos::eos_submission_material::EosSubmissionMaterial, errors::AppError, types::Result};
use std::{fs::read_to_string, path::Path};

pub fn get_sample_eos_on_eth_submission_material_string_n(n: usize) -> Result<String> {
    let path = match n {
        1 => Ok("src/eos_on_eth/eos/eos_test_utils/eos-on-eth-submission-material-1.json"),
        _ => Err(AppError::Custom(format!(
            "Cannot find eos-on-eth submission material num: {}",
            n
        ))),
    }?;
    match Path::new(&path).exists() {
        true => Ok(read_to_string(path)?),
        false => Err("✘ Cannot find sample eos-on-eth-submission-material file!".into()),
    }
}

pub fn get_eos_on_eth_submission_material_n(n: usize) -> Result<EosSubmissionMaterial> {
    EosSubmissionMaterial::from_str(&get_sample_eos_on_eth_submission_material_string_n(n)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_eos_on_eth_submission_material_n() {
        let result = get_eos_on_eth_submission_material_n(1);
        assert!(result.is_ok());
    }
}
