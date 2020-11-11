use std::str::FromStr;
use eos_primitives::AccountName as EosAccountName;
use derive_more::{
    Deref,
    DerefMut,
    Constructor,
};
use crate::{
    btc_on_eos::utils::convert_u64_to_8_decimal_eos_asset,
    types::{
        Byte,
        Bytes,
        Result,
    },
    constants::SAFE_EOS_ADDRESS,
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
