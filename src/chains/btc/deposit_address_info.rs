use std::{
    str::FromStr,
    collections::HashMap,
};
use crate::{
    types::Result,
    errors::AppError,
    utils::strip_hex_prefix,
};
use bitcoin::{
    util::address::Address as BtcAddress,
    hashes::{
        Hash,
        sha256d,
    },
};

pub type DepositInfoList = Vec<DepositAddressInfo>;
pub type DepositAddressJsonList = Vec<DepositAddressInfoJson>;
pub type DepositInfoHashMap =  HashMap<BtcAddress, DepositAddressInfo>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepositAddressListVersion {
    V0,
    V1,
}

impl DepositAddressListVersion {
    pub fn from_maybe_string(maybe_string: &Option<String>) -> Result<Self> {
        match maybe_string {
            None => Ok(DepositAddressListVersion::V0),
            Some(version_string) => DepositAddressListVersion::from_string(version_string.clone()),
        }
    }

    pub fn from_string(version_string: String) -> Result<Self> {
        match version_string.chars().next() {
            Some('0') => Ok(DepositAddressListVersion::V0),
            Some('1') => Ok(DepositAddressListVersion::V1),
            _ => Err(AppError::Custom(format!("✘ Deposit address list version unrecognized: {}", version_string)))
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DepositAddressListVersion::V0 => "0".to_string(),
            DepositAddressListVersion::V1 => "1".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositAddressInfoJson {
    pub nonce: u64,
    pub address: String,
    pub btc_deposit_address: String,
    pub maybe_version: Option<String>,
    pub address_and_nonce_hash: String,
}

impl DepositAddressInfoJson {
    #[cfg(test)]
    pub fn new(
        nonce: u64,
        address: String,
        btc_deposit_address: String,
        address_and_nonce_hash: String,
        maybe_version: Option<String>,
    ) -> Self {
        DepositAddressInfoJson {
            nonce,
            address,
            maybe_version,
            btc_deposit_address,
            address_and_nonce_hash,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepositAddressInfo {
    pub nonce: u64,
    pub address: String,
    pub commitment_hash: sha256d::Hash,
    pub btc_deposit_address: BtcAddress,
    pub version: DepositAddressListVersion,
}

impl DepositAddressInfo {
    pub fn new(
        nonce: u64,
        address: &String,
        btc_deposit_address: &String,
        commitment_hash: &String,
        version: &DepositAddressListVersion,
    ) -> Result<Self> {
        Ok(
            DepositAddressInfo {
                nonce,
                version: version.clone(),
                address: address.to_string(),
                btc_deposit_address: BtcAddress::from_str(&btc_deposit_address)?,
                commitment_hash: sha256d::Hash::from_slice(&hex::decode(strip_hex_prefix(commitment_hash)?)?)?,
            }
        )
    }

    pub fn from_json(deposit_address_info_json: &DepositAddressInfoJson) -> Result<Self> {
        Self::new(
            deposit_address_info_json.nonce.clone(),
            &deposit_address_info_json.address.clone(),
            &deposit_address_info_json.btc_deposit_address.clone(),
            &deposit_address_info_json.address_and_nonce_hash.clone(),
            &DepositAddressListVersion::from_maybe_string(&deposit_address_info_json.maybe_version)?,
        )
    }

    pub fn to_json(&self) -> DepositAddressInfoJson {
        DepositAddressInfoJson {
            nonce: self.nonce,
            address: self.address.clone(),
            maybe_version: Some(self.version.to_string()),
            btc_deposit_address: self.btc_deposit_address.to_string(),
            address_and_nonce_hash: hex::encode(self.commitment_hash),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_deposit_info_json_to_deposit_info() {
        let nonce = 1578079722;
        let address = "0xedb86cd455ef3ca43f0e227e00469c3bdfa40628"
            .to_string();
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ"
            .to_string();
        let address_and_nonce_hash =
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae"
            .to_string();
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            btc_deposit_address,
            address_and_nonce_hash,
            maybe_version: None,
        };
        if let Err(e) = DepositAddressInfo::from_json(&deposit_json) {
            panic!("Error parsing deposit info json: {}", e);
        }
    }
}
