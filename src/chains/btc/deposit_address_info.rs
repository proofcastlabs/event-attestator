use std::{
    str::FromStr,
    collections::HashMap,
};
use crate::{
    types::Result,
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
enum DepositAddressListVersion {
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

    pub fn from_deposit_address_info(deposit_address_info: &DepositAddressInfo) -> Self {
        deposit_address_info.to_json()
    }

    pub fn to_deposit_address_info(&self) -> Result<DepositAddressInfo> {
        DepositAddressInfo::from_json(&self)
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
    pub fn new( // Make this new_eos_info or something
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
