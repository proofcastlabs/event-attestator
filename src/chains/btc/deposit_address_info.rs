use bitcoin::{
    hashes::sha256d,
    util::address::Address as BtcAddress,
};
use std::{
    str::FromStr,
    collections::HashMap,
};
use crate::{
    types::Result,
    errors::AppError,
    chains::btc::btc_utils::convert_hex_to_sha256_hash,
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
    pub address_and_nonce_hash: Option<String>,
    pub eth_address_and_nonce_hash: Option<String>, // NOTE: For legacy reasons.
}

impl DepositAddressInfoJson {
    #[cfg(test)]
    pub fn new(
        nonce: u64,
        address: String,
        btc_deposit_address: String,
        address_and_nonce_hash: String,
        maybe_version: Option<String>,
    ) -> Result<Self> {
        match DepositAddressListVersion::from_maybe_string(&maybe_version)? {
            DepositAddressListVersion::V0 => Ok(DepositAddressInfoJson {
                nonce,
                address,
                maybe_version,
                btc_deposit_address,
                address_and_nonce_hash: None,
                eth_address_and_nonce_hash: Some(address_and_nonce_hash)
            }),
            DepositAddressListVersion::V1 => Ok(DepositAddressInfoJson {
                nonce,
                address,
                maybe_version,
                btc_deposit_address,
                eth_address_and_nonce_hash: None,
                address_and_nonce_hash: Some(address_and_nonce_hash),
            }),
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
    fn extract_address_and_nonce_hash_string_from_json(
        deposit_address_info_json: &DepositAddressInfoJson
    ) -> Result<String> {
        match &deposit_address_info_json.address_and_nonce_hash {
            Some(hash_string) => Ok(hash_string.clone()),
            None => match &deposit_address_info_json.eth_address_and_nonce_hash {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(AppError::Custom(
                    format!(
                        "✘ No address and nonce hash string found in json: {}",
                        serde_json::to_string(deposit_address_info_json)?,
                    )
                )),
            }
        }
    }

    fn extract_address_and_nonce_hash_from_json(
        deposit_address_info_json: &DepositAddressInfoJson
    ) -> Result<sha256d::Hash> {
        Self::extract_address_and_nonce_hash_string_from_json(deposit_address_info_json)
            .and_then(|hex| convert_hex_to_sha256_hash(&hex))
    }

    pub fn from_json(deposit_address_info_json: &DepositAddressInfoJson) -> Result<Self> {
        Ok(DepositAddressInfo {
            nonce: deposit_address_info_json.nonce.clone(),
            address: deposit_address_info_json.address.clone(),
            btc_deposit_address: BtcAddress::from_str(&deposit_address_info_json.btc_deposit_address)?,
            commitment_hash: Self::extract_address_and_nonce_hash_from_json(deposit_address_info_json)?,
            version: DepositAddressListVersion::from_maybe_string(&deposit_address_info_json.maybe_version)?,
        })
    }

    pub fn to_json(&self) -> DepositAddressInfoJson {
        let hash_string = hex::encode(self.commitment_hash);
        DepositAddressInfoJson {
            nonce: self.nonce,
            address: self.address.clone(),
            maybe_version: Some(self.version.to_string()),
            btc_deposit_address: self.btc_deposit_address.to_string(),
            eth_address_and_nonce_hash: match self.version {
                DepositAddressListVersion::V0 => Some(hash_string.clone()),
                DepositAddressListVersion::V1 => None,
            },
            address_and_nonce_hash: match self.version {
                DepositAddressListVersion::V0 => None,
                DepositAddressListVersion::V1 => Some(hash_string),
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_convert_deposit_info_json_to_deposit_info() {
        let nonce = 1578079722;
        let address = "0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string();
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let eth_address_and_nonce_hash = None;
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            btc_deposit_address,
            maybe_version: None,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        if let Err(e) = DepositAddressInfo::from_json(&deposit_json) {
            panic!("Error parsing deposit info json: {}", e);
        }
    }
}
