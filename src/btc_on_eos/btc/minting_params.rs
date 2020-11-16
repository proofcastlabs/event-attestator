use std::str::FromStr;
use eos_primitives::AccountName as EosAccountName;
use derive_more::{
    Deref,
    DerefMut,
    Constructor,
};
use crate::{
    constants::SAFE_EOS_ADDRESS,
    btc_on_eos::utils::convert_eos_asset_to_u64,
    btc_on_eos::utils::convert_u64_to_8_decimal_eos_asset,
    chains::btc::btc_constants::MINIMUM_REQUIRED_SATOSHIS,
    types::{
        Byte,
        Bytes,
        Result,
    },
};
use bitcoin::{
    hashes::sha256d,
    util::address::Address as BtcAddress,
};

#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut, Constructor, Serialize, Deserialize)]
pub struct BtcOnEosMintingParams(pub Vec<BtcOnEosMintingParamStruct>);

impl BtcOnEosMintingParams {
    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }

    pub fn filter_out_value_too_low(&self) -> Result<BtcOnEosMintingParams> {
        info!("✔ Filtering out any minting params below a minimum of {} Satoshis...", MINIMUM_REQUIRED_SATOSHIS);
        Ok(BtcOnEosMintingParams::new(
            self
                .iter()
                .map(|params| convert_eos_asset_to_u64(&params.amount))
                .collect::<Result<Vec<u64>>>()?
                .into_iter()
                .zip(self.iter())
                .filter(|(amount, params)| {
                    match amount >= &MINIMUM_REQUIRED_SATOSHIS {
                        true => true,
                        false => {
                            info!("✘ Filtering minting params ∵ value too low: {:?}", params);
                            false
                        }
                    }
                })
                .map(|(_, params)| params)
                .cloned()
                .collect::<Vec<BtcOnEosMintingParamStruct>>()
        ))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BtcOnEosMintingParamStruct {
    pub amount: String,
    pub to: String,
    pub originating_tx_hash: String,
    pub originating_tx_address: String,
}

impl BtcOnEosMintingParamStruct {
    pub fn new(
        amount: u64,
        to: String,
        originating_tx_hash: sha256d::Hash,
        originating_tx_address: BtcAddress,
        symbol: &str,
    ) -> BtcOnEosMintingParamStruct {
        BtcOnEosMintingParamStruct {
            to: match EosAccountName::from_str(&to) {
                Ok(_) => to,
                Err(_) => {
                    info!("✘ Error converting '{}' to EOS address!", to);
                    info!("✔ Defaulting to safe EOS address: '{}'", SAFE_EOS_ADDRESS);
                    SAFE_EOS_ADDRESS.to_string()
                }
            },
            amount: convert_u64_to_8_decimal_eos_asset(amount, symbol),
            originating_tx_hash: originating_tx_hash.to_string(),
            originating_tx_address: originating_tx_address.to_string(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::btc_on_eos::btc::btc_test_utils::get_sample_minting_params;

    #[test]
    fn should_filter_minting_params() {
        let expected_length_before = 3;
        let expected_length_after = 2;
        let minting_params = get_sample_minting_params();
        let length_before = minting_params.len();
        assert_eq!(length_before, expected_length_before);
        let result = minting_params.filter_out_value_too_low().unwrap();
        let length_after = result.len();
        assert_eq!(length_after, expected_length_after);
        result
            .iter()
            .map(|params| assert!(convert_eos_asset_to_u64(&params.amount).unwrap() >= MINIMUM_REQUIRED_SATOSHIS))
            .for_each(drop);
    }
}
