use crate::{
    chains::btc::btc_utils::convert_hex_to_sha256_hash,
    types::{Bytes, Result},
    utils::decode_hex_with_err_msg,
};
use bitcoin::{
    hashes::{sha256d, Hash},
    util::address::Address as BtcAddress,
};
use derive_more::{Constructor, Deref};
use std::{collections::HashMap, fmt, str::FromStr};

pub type DepositInfoHashMap = HashMap<BtcAddress, DepositAddressInfo>;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Deref, Constructor)]
pub struct DepositAddressInfoJsonList(pub Vec<DepositAddressInfoJson>);

#[derive(Clone, Debug, PartialEq, Eq, Deref, Constructor)]
pub struct DepositInfoList(pub Vec<DepositAddressInfo>);

impl DepositInfoList {
    pub fn from_json(json: &DepositAddressInfoJsonList) -> Result<Self> {
        Ok(Self::new(
            json.iter()
                .map(DepositAddressInfo::from_json)
                .collect::<Result<Vec<DepositAddressInfo>>>()?,
        ))
    }

    pub fn validate(&self) -> Result<()> {
        self.iter().map(|info| info.validate()).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepositAddressInfoVersion {
    V0,
    V1,
    V2,
}

impl DepositAddressInfoVersion {
    pub fn from_maybe_string(maybe_string: &Option<String>) -> Result<Self> {
        match maybe_string {
            None => Ok(DepositAddressInfoVersion::V0),
            Some(version_string) => DepositAddressInfoVersion::from_string(&version_string),
        }
    }

    pub fn from_string(version_string: &str) -> Result<Self> {
        match version_string.chars().next() {
            Some('0') => Ok(DepositAddressInfoVersion::V0),
            Some('1') => Ok(DepositAddressInfoVersion::V1),
            Some('2') => Ok(DepositAddressInfoVersion::V2),
            _ => Err(format!("✘ Deposit address list version unrecognized: {}", version_string).into()),
        }
    }
}

impl fmt::Display for DepositAddressInfoVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DepositAddressInfoVersion::V0 => write!(f, "0"),
            DepositAddressInfoVersion::V1 => write!(f, "1"),
            DepositAddressInfoVersion::V2 => write!(f, "2"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositAddressInfoJson {
    pub nonce: u64,
    pub address: Option<String>,
    pub version: Option<String>,
    pub calldata: Option<String>,
    pub eth_address: Option<String>, // NOTE: For legacy reasons.
    pub btc_deposit_address: String,
    pub address_and_nonce_hash: Option<String>,
    pub eth_address_and_nonce_hash: Option<String>, // NOTE: Ibid.
}

impl DepositAddressInfoJson {
    pub fn to_string(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    #[cfg(test)]
    pub fn from_str(json_string: &str) -> Result<Self> {
        Ok(serde_json::from_str(&json_string)?)
    }
}

#[cfg(test)]
use crate::types::Byte;
#[cfg(test)]
impl DepositAddressInfoJson {
    pub fn new(
        nonce: u64,
        address: String,
        btc_deposit_address: String,
        address_and_nonce_hash: String,
        version: Option<String>,
        calldata: &[Byte],
    ) -> Result<Self> {
        match DepositAddressInfoVersion::from_maybe_string(&version)? {
            DepositAddressInfoVersion::V0 => Ok(DepositAddressInfoJson {
                nonce,
                version,
                address: None,
                calldata: None,
                btc_deposit_address,
                eth_address: Some(address),
                address_and_nonce_hash: None,
                eth_address_and_nonce_hash: Some(address_and_nonce_hash),
            }),
            DepositAddressInfoVersion::V1 => Ok(DepositAddressInfoJson {
                nonce,
                version,
                calldata: None,
                eth_address: None,
                btc_deposit_address,
                address: Some(address),
                eth_address_and_nonce_hash: None,
                address_and_nonce_hash: Some(address_and_nonce_hash),
            }),
            DepositAddressInfoVersion::V2 => Ok(DepositAddressInfoJson {
                nonce,
                version,
                eth_address: None,
                btc_deposit_address,
                address: Some(address),
                eth_address_and_nonce_hash: None,
                calldata: Some(hex::encode(&calldata)),
                address_and_nonce_hash: Some(address_and_nonce_hash),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DepositAddressInfo {
    pub nonce: u64,
    pub address: String,
    pub calldata: Bytes,
    pub commitment_hash: sha256d::Hash,
    pub btc_deposit_address: BtcAddress,
    pub version: DepositAddressInfoVersion,
}

impl DepositAddressInfo {
    fn get_missing_field_err_msg(field_name: &str) -> String {
        format!("✘ No '{}' field in deposit address info json!", field_name)
    }

    fn extract_address_and_nonce_hash_string_from_json(
        deposit_address_info_json: &DepositAddressInfoJson,
    ) -> Result<String> {
        match DepositAddressInfoVersion::from_maybe_string(&deposit_address_info_json.version)? {
            DepositAddressInfoVersion::V0 => match &deposit_address_info_json.eth_address_and_nonce_hash {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(Self::get_missing_field_err_msg("eth_address_and_nonce_hash").into()),
            },
            DepositAddressInfoVersion::V1 | DepositAddressInfoVersion::V2 => {
                match &deposit_address_info_json.address_and_nonce_hash {
                    Some(hash_string) => Ok(hash_string.clone()),
                    None => Err(Self::get_missing_field_err_msg("address_and_nonce_hash").into()),
                }
            },
        }
    }

    fn extract_address_and_nonce_hash_from_json(
        deposit_address_info_json: &DepositAddressInfoJson,
    ) -> Result<sha256d::Hash> {
        Self::extract_address_and_nonce_hash_string_from_json(deposit_address_info_json)
            .and_then(|hex| convert_hex_to_sha256_hash(&hex))
    }

    fn extract_address_string_from_json(deposit_address_info_json: &DepositAddressInfoJson) -> Result<String> {
        match DepositAddressInfoVersion::from_maybe_string(&deposit_address_info_json.version)? {
            DepositAddressInfoVersion::V0 => match &deposit_address_info_json.eth_address {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(Self::get_missing_field_err_msg("eth_address").into()),
            },
            DepositAddressInfoVersion::V1 | DepositAddressInfoVersion::V2 => match &deposit_address_info_json.address {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(Self::get_missing_field_err_msg("address").into()),
            },
        }
    }

    pub fn from_json(deposit_address_info_json: &DepositAddressInfoJson) -> Result<Self> {
        Ok(DepositAddressInfo {
            nonce: deposit_address_info_json.nonce,
            address: Self::extract_address_string_from_json(deposit_address_info_json)?,
            btc_deposit_address: BtcAddress::from_str(&deposit_address_info_json.btc_deposit_address)?,
            commitment_hash: Self::extract_address_and_nonce_hash_from_json(deposit_address_info_json)?,
            version: DepositAddressInfoVersion::from_maybe_string(&deposit_address_info_json.version)?,
            calldata: match &deposit_address_info_json.calldata {
                Some(hex_string) => decode_hex_with_err_msg(
                    hex_string,
                    &format!("✘ Could not decode hex in calldata in {}: ", deposit_address_info_json.to_string()?),
                )?,
                None => vec![],
            },
        })
    }

    fn get_address_as_bytes(&self) -> Result<Bytes> {
        match self.version {
            DepositAddressInfoVersion::V1 => Ok(self.address.as_bytes().to_vec()),
            DepositAddressInfoVersion::V0 | DepositAddressInfoVersion::V2 => decode_hex_with_err_msg(
                &self.address,
                &format!("✘ Could not decode address hex in {}: ", self.to_json().to_string()?),
            ),
        }
    }

    fn calculate_commitment_hash_v0(&self) -> Result<sha256d::Hash> {
        self.get_address_as_bytes().map(|mut address_bytes| {
            address_bytes.append(&mut self.nonce.to_le_bytes().to_vec());
            sha256d::Hash::hash(&address_bytes)
        })
    }

    fn calculate_commitment_hash_v1(&self) -> Result<sha256d::Hash> {
        self.calculate_commitment_hash_v0()
    }

    fn calculate_commitment_hash_v2(&self) -> Result<sha256d::Hash> {
        self.get_address_as_bytes().map(|mut address_bytes| {
            address_bytes.append(&mut self.nonce.to_le_bytes().to_vec());
            address_bytes.append(&mut self.calldata.clone());
            sha256d::Hash::hash(&address_bytes)
        })
    }

    fn calculate_commitment_hash(&self) -> Result<sha256d::Hash> {
        match self.version {
            DepositAddressInfoVersion::V0 => self.calculate_commitment_hash_v0(),
            DepositAddressInfoVersion::V1 => self.calculate_commitment_hash_v1(),
            DepositAddressInfoVersion::V2 => self.calculate_commitment_hash_v2(),
        }
    }

    fn validate_commitment_hash(&self) -> Result<()> {
        self.calculate_commitment_hash()
            .and_then(|calculated_hash| match calculated_hash == self.commitment_hash {
                true => Ok(()),
                false => {
                    debug!("          Deposit info nonce: {}", &self.nonce);
                    debug!("        Deposit info adresss: {}", &self.address);
                    debug!("  Calculated commitment hash: {}", &calculated_hash);
                    debug!("Deposit info commitment hash: {}", &self.commitment_hash);
                    Err("✘ Deposit info error - commitment hash is not valid!".into())
                },
            })
    }

    pub fn to_json(&self) -> DepositAddressInfoJson {
        let hash_string = hex::encode(self.commitment_hash);
        DepositAddressInfoJson {
            nonce: self.nonce,
            version: Some(self.version.to_string()),
            btc_deposit_address: self.btc_deposit_address.to_string(),
            calldata: match self.version {
                DepositAddressInfoVersion::V0 | DepositAddressInfoVersion::V1 => None,
                DepositAddressInfoVersion::V2 => Some(hex::encode(&self.calldata)),
            },
            address: match self.version {
                DepositAddressInfoVersion::V0 => None,
                DepositAddressInfoVersion::V1 | DepositAddressInfoVersion::V2 => Some(self.address.clone()),
            },
            eth_address: match self.version {
                DepositAddressInfoVersion::V0 => Some(self.address.clone()),
                DepositAddressInfoVersion::V1 | DepositAddressInfoVersion::V2 => None,
            },
            eth_address_and_nonce_hash: match self.version {
                DepositAddressInfoVersion::V0 => Some(hash_string.clone()),
                DepositAddressInfoVersion::V1 | DepositAddressInfoVersion::V2 => None,
            },
            address_and_nonce_hash: match self.version {
                DepositAddressInfoVersion::V0 => None,
                DepositAddressInfoVersion::V1 | DepositAddressInfoVersion::V2 => Some(hash_string),
            },
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.validate_commitment_hash()
    }

    #[cfg(test)]
    pub fn from_str(s: &str) -> Result<Self> {
        Self::from_json(&DepositAddressInfoJson::from_str(s)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;

    fn get_sample_deposit_info_json_string_v0() -> String {
        "{\"btc_deposit_address\":\"2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2\",\"eth_address\":\"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac\",\"eth_address_and_nonce_hash\":\"0x98eaf3812c998a46e0ee997ccdadf736c7bc13c18a5292df7a8d39089fd28d9e\",\"nonce\":1337,\"public_key\":\"03d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7\",\"version\":\"0\"}".to_string()
    }

    fn get_sample_deposit_info_json_string_v1() -> String {
        "{\"address\":\"0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC\",\"address_and_nonce_hash\":\"0x5364a60af6f1e0e8a0b0e38b8812e3c02b98727247d749500ee1e90066aa360e\",\"btc_deposit_address\":\"2NEqdGbbaHdCUBbSHRBgFVPNjgw3Gnt1zm5\",\"nonce\":1337,\"public_key\":\"03d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7\",\"version\":\"1\"}".to_string()
    }

    fn get_sample_deposit_info_json_string_v2() -> String {
        "{\"address\":\"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac\",\"address_and_nonce_hash\":\"0x693777b55c79e66153181b67faa43662be576e5896003444d0479fe9b7a23d38\",\"btc_deposit_address\":\"2NFHg6i6R5N29MB7B1oK7PsLZhqRg456rWD\",\"calldata\":\"0x404092\",\"nonce\":1337,\"public_key\":\"03d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7\",\"version\":\"2\"}".to_string()
    }

    fn get_sample_deposit_info_v0() -> DepositAddressInfo {
        DepositAddressInfo::from_str(&get_sample_deposit_info_json_string_v0()).unwrap()
    }

    fn get_sample_deposit_info_v1() -> DepositAddressInfo {
        DepositAddressInfo::from_str(&get_sample_deposit_info_json_string_v1()).unwrap()
    }

    fn get_sample_deposit_info_v2() -> DepositAddressInfo {
        DepositAddressInfo::from_str(&get_sample_deposit_info_json_string_v2()).unwrap()
    }

    #[test]
    fn should_err_if_json_is_v1_and_has_no_address_and_nonce_hash_key() {
        let nonce = 1578079722;
        let address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2MuuCeJjptiB1ETfytAqMZFqPCKAfXyhxoQ".to_string();
        let eth_address_and_nonce_hash =
            Some("348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string());
        let eth_address = None;
        let address_and_nonce_hash = None;
        let calldata = None;
        let version = Some("1".to_string());
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            calldata,
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
        let eth_address_and_nonce_hash =
            Some("348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string());
        let calldata = None;
        let eth_address = None;
        let address_and_nonce_hash = None;
        let version = Some("0".to_string());
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            calldata,
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
        let address_and_nonce_hash =
            Some("348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string());
        let address = None;
        let calldata = None;
        let version = Some("1".to_string());
        let eth_address_and_nonce_hash = None;
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            eth_address,
            calldata,
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
        let address_and_nonce_hash =
            Some("348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string());
        let address = None;
        let eth_address_and_nonce_hash = None;
        let version = Some("0".to_string());
        let calldata = None;
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            calldata,
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
        let eth_address_and_nonce_hash =
            Some("348c7ab8078c400c5b07d1c3dda4fff8218bb6f2dc40f72662edc13ed867fcae".to_string());
        let version = None;
        let address = None;
        let calldata = None;
        let address_and_nonce_hash = None;
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            calldata,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let result = DepositAddressInfo::from_json(&deposit_json).unwrap();
        assert_eq!(result.version, DepositAddressInfoVersion::V0);
    }

    #[test]
    fn should_convert_valid_deposit_info_json_to_deposit_info() {
        let nonce = 1578079722;
        let address = Some("0xedb86cd455ef3ca43f0e227e00469c3bdfa40628".to_string());
        let btc_deposit_address = "2NCbnp5Lp1eNeT9iBz9UrjwKCTUeQtjEcyy".to_string();
        let address_and_nonce_hash =
            Some("0x5b455d06e29f2b65279b947304f03ebb327cbf7d3fb2d7cd488a27c1bbf00ae9".to_string());
        let calldata = None;
        let eth_address = None;
        let eth_address_and_nonce_hash = None;
        let version = Some("1.0.0".to_string());
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            calldata,
            eth_address,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        if let Err(e) = DepositAddressInfo::from_json(&deposit_json) {
            panic!("Error parsing deposit info json: {}", e);
        }
    }

    #[test]
    fn should_convert_v0_deposit_info_string_to_deposit_info() {
        let json_str = get_sample_deposit_info_json_string_v0();
        let result = DepositAddressInfo::from_str(&json_str);
        assert!(result.is_ok());
    }

    #[test]
    fn should_convert_v1_deposit_info_string_to_deposit_info() {
        let json_str = get_sample_deposit_info_json_string_v1();
        let result = DepositAddressInfo::from_str(&json_str);
        assert!(result.is_ok());
    }

    #[test]
    fn should_convert_v2_deposit_info_string_to_deposit_info() {
        let json_str = get_sample_deposit_info_json_string_v2();
        let result = DepositAddressInfo::from_str(&json_str);
        assert!(result.is_ok())
    }

    #[test]
    fn v0_deposit_info_should_be_valid() {
        let deposit_info = get_sample_deposit_info_v0();
        let result = deposit_info.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn v1_deposit_info_should_be_valid() {
        let deposit_info = get_sample_deposit_info_v1();
        let result = deposit_info.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn v2_deposit_info_should_be_valid() {
        let deposit_info = get_sample_deposit_info_v2();
        let result = deposit_info.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn v0_invalid_commitment_hash_should_fail_validation() {
        let mut deposit_info = get_sample_deposit_info_v0();
        let expected_err = "✘ Deposit info error - commitment hash is not valid!".to_string();
        deposit_info.nonce = deposit_info.nonce + 1;
        match deposit_info.validate() {
            Ok(_) => panic!("Should not be valid!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn v1_invalid_commitment_hash_should_fail_validation() {
        let mut deposit_info = get_sample_deposit_info_v1();
        let expected_err = "✘ Deposit info error - commitment hash is not valid!".to_string();
        deposit_info.nonce = deposit_info.nonce + 1;
        match deposit_info.validate() {
            Ok(_) => panic!("Should not be valid!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(_) => panic!("Wrong error received!"),
        }
    }

    #[test]
    fn v2_invalid_commitment_hash_should_fail_validation() {
        let mut deposit_info = get_sample_deposit_info_v2();
        let expected_err = "✘ Deposit info error - commitment hash is not valid!".to_string();
        deposit_info.nonce = deposit_info.nonce + 1;
        match deposit_info.validate() {
            Ok(_) => panic!("Should not be valid!"),
            Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
