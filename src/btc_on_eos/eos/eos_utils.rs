use std::str::FromStr;
use eos_primitives::AccountName as EosAccountName;
use crate::btc_on_eos::{
    errors::AppError,
    constants::SAFE_EOS_ADDRESS,
    eos::eos_constants::EOS_SCHEDULE_DB_PREFIX,
    types::{
        Bytes,
        Result,
    },
};

pub fn get_eos_schedule_db_key(version: u32) -> Bytes {
    format!("{}{}", EOS_SCHEDULE_DB_PREFIX, version).as_bytes().to_vec()
}

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
