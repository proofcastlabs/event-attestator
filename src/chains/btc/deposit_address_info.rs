use bitcoin::{
    util::address::Address as BtcAddress,
    hashes::{
        Hash,
        sha256d,
    },
};
use std::{
    str::FromStr,
    collections::HashMap,
};
use crate::{
    errors::AppError,
    chains::btc::btc_utils::convert_hex_to_sha256_hash,
    types::{
        Bytes,
        Result,
    },
};

pub type DepositInfoList = Vec<DepositAddressInfo>;
pub type DepositAddressJsonList = Vec<DepositAddressInfoJson>;
pub type DepositInfoHashMap =  HashMap<BtcAddress, DepositAddressInfo>;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepositAddressInfoVersion {
    V0,
    V1,
}

impl DepositAddressInfoVersion {
    pub fn from_maybe_string(maybe_string: &Option<String>) -> Result<Self> {
        match maybe_string {
            None => Ok(DepositAddressInfoVersion::V0),
            Some(version_string) => DepositAddressInfoVersion::from_string(version_string.clone()),
        }
    }

    pub fn from_string(version_string: String) -> Result<Self> {
        match version_string.chars().next() {
            Some('0') => Ok(DepositAddressInfoVersion::V0),
            Some('1') => Ok(DepositAddressInfoVersion::V1),
            _ => Err(AppError::Custom(format!("✘ Deposit address list version unrecognized: {}", version_string)))
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DepositAddressInfoVersion::V0 => "0".to_string(),
            DepositAddressInfoVersion::V1 => "1".to_string(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositAddressInfoJson {
    pub nonce: u64,
    pub address: Option<String>,
    pub version: Option<String>,
    pub eth_address: Option<String>, // NOTE: For legacy reasons.
    pub btc_deposit_address: String,
    pub address_and_nonce_hash: Option<String>,
    pub eth_address_and_nonce_hash: Option<String>, // NOTE: Ibid.
}

impl DepositAddressInfoJson {
    #[cfg(test)]
    pub fn new(
        nonce: u64,
        address: String,
        btc_deposit_address: String,
        address_and_nonce_hash: String,
        version: Option<String>,
    ) -> Result<Self> {
        match DepositAddressInfoVersion::from_maybe_string(&version)? {
            DepositAddressInfoVersion::V0 => Ok(DepositAddressInfoJson {
                nonce,
                version,
                address: None,
                btc_deposit_address,
                eth_address: Some(address),
                address_and_nonce_hash: None,
                eth_address_and_nonce_hash: Some(address_and_nonce_hash)
            }),
            DepositAddressInfoVersion::V1 => Ok(DepositAddressInfoJson {
                nonce,
                version,
                eth_address: None,
                btc_deposit_address,
                address: Some(address),
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
    pub version: DepositAddressInfoVersion,
}

impl DepositAddressInfo {
    fn get_missing_field_err_msg(field_name: &str) -> String {
        format!("✘ No '{}' field in deposit address info json!", field_name)
    }

    fn extract_address_and_nonce_hash_string_from_json(
        deposit_address_info_json: &DepositAddressInfoJson
    ) -> Result<String> {
        match DepositAddressInfoVersion::from_maybe_string(&deposit_address_info_json.version)? {
            DepositAddressInfoVersion::V0 => match &deposit_address_info_json.eth_address_and_nonce_hash {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(AppError::Custom(Self::get_missing_field_err_msg("eth_address_and_nonce_hash"))),
            },
            DepositAddressInfoVersion::V1 => match &deposit_address_info_json.address_and_nonce_hash {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(AppError::Custom(Self::get_missing_field_err_msg("address_and_nonce_hash"))),
            },
        }
    }

    fn extract_address_and_nonce_hash_from_json(
        deposit_address_info_json: &DepositAddressInfoJson
    ) -> Result<sha256d::Hash> {
        Self::extract_address_and_nonce_hash_string_from_json(deposit_address_info_json)
            .and_then(|hex| convert_hex_to_sha256_hash(&hex))
    }

    fn extract_address_string_from_json(deposit_address_info_json: &DepositAddressInfoJson) -> Result<String> {
        match DepositAddressInfoVersion::from_maybe_string(&deposit_address_info_json.version)? {
            DepositAddressInfoVersion::V0 => match &deposit_address_info_json.eth_address {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(AppError::Custom(Self::get_missing_field_err_msg("eth_address"))),
            },
            DepositAddressInfoVersion::V1 => match &deposit_address_info_json.address {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(AppError::Custom(Self::get_missing_field_err_msg("address"))),
            }
        }
    }

    fn from_json_with_no_validation(deposit_address_info_json: &DepositAddressInfoJson) -> Result<Self> {
        Ok(
            DepositAddressInfo {
                nonce: deposit_address_info_json.nonce.clone(),
                address: Self::extract_address_string_from_json(deposit_address_info_json)?,
                btc_deposit_address: BtcAddress::from_str(&deposit_address_info_json.btc_deposit_address)?,
                commitment_hash: Self::extract_address_and_nonce_hash_from_json(deposit_address_info_json)?,
                version: DepositAddressInfoVersion::from_maybe_string(&deposit_address_info_json.version)?,
            }
        )
    }

    fn get_address_as_bytes(&self) -> Result<Bytes> {
        match self.version {
            DepositAddressInfoVersion::V0 => Ok(hex::decode(&self.address[..].replace("0x", ""))?),
            DepositAddressInfoVersion::V1 => Ok(self.address.as_bytes().to_vec()),
        }
    }

    fn calculate_commitment_hash(&self) -> Result<sha256d::Hash> {
        self.get_address_as_bytes()
            .and_then(|mut address_bytes| {
                address_bytes.append(&mut self.nonce.to_le_bytes().to_vec());
                Ok(sha256d::Hash::hash(&address_bytes))
            })
    }

    fn validate_commitment_hash(self) -> Result<Self> {
        self.calculate_commitment_hash()
            .and_then(|calculated_hash| {
                match calculated_hash == self.commitment_hash {
                    true => Ok(self),
                    false => {
                        debug!("          Deposit info nonce: {}", &self.nonce);
                        debug!("        Deposit info adresss: {}", &self.address);
                        debug!("  Calculated commitment hash: {}", &calculated_hash);
                        debug!("Deposit info commitment hash: {}", &self.commitment_hash);
                        Err(AppError::Custom("✘ Deposit info error - commitment hash is not valid!".to_string()))
                    },
                }
            })
    }

    pub fn from_json(deposit_address_info_json: &DepositAddressInfoJson) -> Result<Self> {
        Self::from_json_with_no_validation(deposit_address_info_json)
            .and_then(DepositAddressInfo::validate_commitment_hash)
    }

    pub fn to_json(&self) -> DepositAddressInfoJson {
        let hash_string = hex::encode(self.commitment_hash);
        DepositAddressInfoJson {
            nonce: self.nonce,
            version: Some(self.version.to_string()),
            btc_deposit_address: self.btc_deposit_address.to_string(),
            address: match self.version {
                DepositAddressInfoVersion::V0 => None,
                DepositAddressInfoVersion::V1 => Some(self.address.clone()),
            },
            eth_address: match self.version {
                DepositAddressInfoVersion::V0 => Some(self.address.clone()),
                DepositAddressInfoVersion::V1 => None,
            },
            eth_address_and_nonce_hash: match self.version {
                DepositAddressInfoVersion::V0 => Some(hash_string.clone()),
                DepositAddressInfoVersion::V1 => None,
            },
            address_and_nonce_hash: match self.version {
                DepositAddressInfoVersion::V0 => None,
                DepositAddressInfoVersion::V1 => Some(hash_string),
            },
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_err_if_json_is_v1_and_has_no_address_and_nonce_hash_key() {
        let nonce = 1578079722;
        let address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let eth_address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let eth_address = None;
        let address_and_nonce_hash = None;
        let version = Some("1".to_string());
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let expected_error = "✘ No 'address_and_nonce_hash' field in deposit address info json!".to_string();
        match DepositAddressInfo::from_json(&deposit_json) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error received: {}", e),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }

    #[test]
    fn should_err_if_json_is_v0_and_has_no_eth_address_field() {
        let nonce = 1578079722;
        let address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let eth_address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let eth_address = None;
        let address_and_nonce_hash = None;
        let version = Some("0".to_string());
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let expected_error = "✘ No 'eth_address' field in deposit address info json!".to_string();
        match DepositAddressInfo::from_json(&deposit_json) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error received: {}", e),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }

    #[test]
    fn should_err_if_json_is_v1_and_has_no_address_field() {
        let nonce = 1578079722;
        let eth_address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let address = None;
        let eth_address_and_nonce_hash = None;
        let version = Some("1".to_string());
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let expected_error = "✘ No 'address' field in deposit address info json!".to_string();
        match DepositAddressInfo::from_json(&deposit_json) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error received: {}", e),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }

    #[test]
    fn should_err_if_json_is_v0_and_has_no_eth_address_and_nonce_hash() {
        let nonce = 1578079722;
        let eth_address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let address = None;
        let eth_address_and_nonce_hash = None;
        let version = Some("0".to_string());
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let expected_error = "✘ No 'eth_address_and_nonce_hash' field in deposit address info json!".to_string();
        match DepositAddressInfo::from_json(&deposit_json) {
            Err(AppError::Custom(e)) => assert_eq!(e, expected_error),
            Err(e) => panic!("Wrong error received: {}", e),
            Ok(_) => panic!("Should not have succeeded!"),
        }
    }

    #[test]
    fn deposit_info_should_be_v0_if_version_field_missing() {
        let nonce = 1578079722;
        let eth_address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let eth_address_and_nonce_hash = Some(
            "348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string()
        );
        let version = None;
        let address = None;
        let address_and_nonce_hash = None;
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let result = DepositAddressInfo::from_json(&deposit_json).unwrap();
        assert_eq!(result.version, DepositAddressInfoVersion::V0);
    }

    #[test]
    fn should_convert_deposit_info_json_to_deposit_info() {
        let nonce = 1578079722;
        let address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2NCbnp5Lp1eNeT9iBz9UrjwKCTUeQtjEcyy".to_string();
        let address_and_nonce_hash = Some(
            "0x5b455d06e29f2b65279b947304f03ebb327cbf7d3fb2d7cd488a27c1bbf00ae9".to_string()
        );
        let eth_address = None;
        let eth_address_and_nonce_hash = None;
        let version = Some("1.0.0".to_string());
        let deposit_json = DepositAddressInfoJson  {
            nonce,
            address,
            version,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        if let Err(e) = DepositAddressInfo::from_json(&deposit_json) {
            panic!("Error parsing deposit info json: {}", e);
        }
    }
}
