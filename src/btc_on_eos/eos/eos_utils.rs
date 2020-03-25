use std::str::FromStr;
use eos_primitives::AccountName as EosAccountName;
use crate::btc_on_eos::{
    types::Result,
    errors::AppError,
    constants::SAFE_EOS_ADDRESS,
};

pub fn safely_convert_string_to_eos_account(
    maybe_account_name: &String
) -> Result<EosAccountName> {
    info!("✔ Converting string to EOS account name...");
    match EosAccountName::from_str(maybe_account_name) {
        Ok(eos_account_name) => {
            Ok(eos_account_name)
        }
        Err(_) => {
            match EosAccountName::from_str(SAFE_EOS_ADDRESS) {
                Ok(eos_account_name) => {
                    info!(
                        "✘ Conversion failed - defaulting to safe address: {}",
                        SAFE_EOS_ADDRESS,
                    );
                    Ok(eos_account_name)
                }
                Err(err) => Err(
                    AppError::Custom(
                        format!(
                            "✘ Error converting safe EOS address: {}",
                            err,
                        )
                    )
                )
            }
        }
    }
}
