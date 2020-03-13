use std::str::FromStr;
use eos_primitives::AccountName as EosAccountName;
use crate::btc_on_eos::{
    errors::AppError,
    eos::eos_types::EosNetwork,
    constants::SAFE_EOS_ADDRESS,
    types::{
        Bytes,
        Result,
    },
    utils::{
        convert_usize_to_bytes,
        convert_bytes_to_usize,
    },
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
                Err(e) => Err(
                    AppError::Custom(
                        "✘ Error converting safe EOS address!".to_string()
                    )
                )
            }
        }
    }
}

pub fn convert_eos_network_to_bytes(network: &EosNetwork) -> Result<Bytes> {
    match network {
        EosNetwork::Mainnet => Ok(convert_usize_to_bytes(&0)),
        EosNetwork::Testnet => Ok(convert_usize_to_bytes(&1)),
    }
}

pub fn convert_bytes_to_eos_network(bytes: &Bytes) -> Result<EosNetwork> {
    match convert_bytes_to_usize(bytes)? {
        1 => Ok(EosNetwork::Testnet),
        _ => Ok(EosNetwork::Mainnet),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serde_eos_network() {
        let eos_network = EosNetwork::Mainnet;
        let bytes = convert_eos_network_to_bytes(&eos_network)
            .unwrap();
        let result = convert_bytes_to_eos_network(&bytes)
            .unwrap();
        assert_eq!(result, eos_network);
    }
}
