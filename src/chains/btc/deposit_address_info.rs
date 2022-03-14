use std::{collections::HashMap, fmt, str::FromStr};

use bitcoin::{
    hashes::{sha256d, Hash},
    network::constants::Network as BtcNetwork,
    util::address::Address as BtcAddress,
};
use derive_more::{Constructor, Deref};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    chains::btc::{
        btc_state::BtcState,
        btc_types::BtcPubKeySlice,
        btc_utils::{convert_hex_to_sha256_hash, get_p2sh_redeem_script_sig},
    },
    metadata::metadata_chain_id::MetadataChainId,
    traits::DatabaseInterface,
    types::{Byte, Bytes, Result},
    utils::{decode_hex_with_err_msg, strip_hex_prefix},
};

pub type DepositInfoHashMap = HashMap<BtcAddress, DepositAddressInfo>;

#[derive(Clone, Debug, PartialEq, Default, Eq, Deserialize, Deref, Constructor)]
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

    pub fn validate(&self, btc_pub_key: &BtcPubKeySlice, network: &BtcNetwork) -> Result<()> {
        self.iter().try_for_each(|info| info.validate(btc_pub_key, network))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepositAddressInfoVersion {
    V0,
    V1,
    V2,
    V3,
}

impl DepositAddressInfoVersion {
    pub fn from_maybe_string(maybe_string: &Option<String>) -> Result<Self> {
        match maybe_string {
            None => Ok(DepositAddressInfoVersion::V0),
            Some(version_string) => DepositAddressInfoVersion::from_string(version_string),
        }
    }

    pub fn from_string(version_string: &str) -> Result<Self> {
        match version_string.chars().next() {
            Some('0') => Ok(DepositAddressInfoVersion::V0),
            Some('1') => Ok(DepositAddressInfoVersion::V1),
            Some('2') => Ok(DepositAddressInfoVersion::V2),
            Some('3') => Ok(DepositAddressInfoVersion::V3),
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
            DepositAddressInfoVersion::V3 => write!(f, "3"),
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositAddressInfoJson {
    pub nonce: u64,
    pub address: Option<String>,
    pub version: Option<String>,
    pub user_data: Option<String>,
    pub eth_address: Option<String>, // NOTE: For legacy reasons.
    pub btc_deposit_address: String,
    pub chain_id_hex: Option<String>,
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
impl DepositAddressInfoJson {
    pub fn new(
        nonce: u64,
        address: String,
        btc_deposit_address: String,
        address_and_nonce_hash: String,
        version: Option<String>,
        user_data: &[Byte],
        chain_id_hex: Option<String>,
    ) -> Result<Self> {
        match DepositAddressInfoVersion::from_maybe_string(&version)? {
            DepositAddressInfoVersion::V0 => Ok(Self {
                nonce,
                version,
                address: None,
                user_data: None,
                btc_deposit_address,
                eth_address: Some(address),
                chain_id_hex,
                address_and_nonce_hash: None,
                eth_address_and_nonce_hash: Some(address_and_nonce_hash),
            }),
            DepositAddressInfoVersion::V1 => Ok(Self {
                nonce,
                version,
                user_data: None,
                eth_address: None,
                btc_deposit_address,
                address: Some(address),
                chain_id_hex,
                eth_address_and_nonce_hash: None,
                address_and_nonce_hash: Some(address_and_nonce_hash),
            }),
            DepositAddressInfoVersion::V2 | DepositAddressInfoVersion::V3 => Ok(Self {
                nonce,
                version,
                eth_address: None,
                btc_deposit_address,
                address: Some(address),
                chain_id_hex,
                eth_address_and_nonce_hash: None,
                address_and_nonce_hash: Some(address_and_nonce_hash),
                user_data: Some(format!("0x{}", hex::encode(&user_data))),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepositAddressInfo {
    pub nonce: u64,
    pub address: String,
    pub chain_id: Bytes,
    pub user_data: Bytes,
    pub commitment_hash: sha256d::Hash,
    pub btc_deposit_address: BtcAddress,
    pub version: DepositAddressInfoVersion,
}

impl DepositAddressInfo {
    fn convert_nonce_to_bytes(nonce: u64) -> Bytes {
        nonce.to_le_bytes().to_vec()
    }

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
            _ => match &deposit_address_info_json.address_and_nonce_hash {
                Some(hash_string) => Ok(hash_string.clone()),
                None => Err(Self::get_missing_field_err_msg("address_and_nonce_hash").into()),
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
            _ => match &deposit_address_info_json.address {
                Some(s) => Ok(s.clone()),
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
            user_data: match &deposit_address_info_json.user_data {
                Some(hex_string) => decode_hex_with_err_msg(
                    hex_string,
                    &format!(
                        "✘ Could not decode hex in user_data in {}: ",
                        deposit_address_info_json.to_string()?
                    ),
                )?,
                None => vec![],
            },
            chain_id: match &deposit_address_info_json.chain_id_hex {
                Some(hex) => hex::decode(strip_hex_prefix(hex))?,
                None => vec![],
            },
        })
    }

    fn get_address_as_bytes(&self) -> Result<Bytes> {
        match self.version {
            DepositAddressInfoVersion::V1 | DepositAddressInfoVersion::V3 => Ok(self.address.as_bytes().to_vec()),
            DepositAddressInfoVersion::V0 | DepositAddressInfoVersion::V2 => decode_hex_with_err_msg(
                &self.address,
                &format!("✘ Could not decode address hex in {}: ", self.to_json().to_string()?),
            ),
        }
    }

    fn calculate_commitment_hash_v0(&self) -> Result<sha256d::Hash> {
        self.get_address_as_bytes().map(|mut address_bytes| {
            address_bytes.append(&mut Self::convert_nonce_to_bytes(self.nonce));
            sha256d::Hash::hash(&address_bytes)
        })
    }

    fn calculate_commitment_hash_v1(&self) -> Result<sha256d::Hash> {
        self.calculate_commitment_hash_v0()
    }

    fn calculate_commitment_hash_v2(&self) -> Result<sha256d::Hash> {
        self.get_address_as_bytes().map(|mut address_bytes| {
            address_bytes.append(&mut Self::convert_nonce_to_bytes(self.nonce));
            address_bytes.append(&mut self.chain_id.clone());
            address_bytes.append(&mut self.user_data.clone());
            sha256d::Hash::hash(&address_bytes)
        })
    }

    fn calculate_commitment_hash_v3(&self) -> Result<sha256d::Hash> {
        self.calculate_commitment_hash_v2()
    }

    fn calculate_commitment_hash(&self) -> Result<sha256d::Hash> {
        match self.version {
            DepositAddressInfoVersion::V0 => self.calculate_commitment_hash_v0(),
            DepositAddressInfoVersion::V1 => self.calculate_commitment_hash_v1(),
            DepositAddressInfoVersion::V2 => self.calculate_commitment_hash_v2(),
            DepositAddressInfoVersion::V3 => self.calculate_commitment_hash_v3(),
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
            user_data: match self.version {
                DepositAddressInfoVersion::V0 | DepositAddressInfoVersion::V1 => None,
                _ => Some(hex::encode(&self.user_data)),
            },
            address: match self.version {
                DepositAddressInfoVersion::V0 => None,
                _ => Some(self.address.clone()),
            },
            eth_address: match self.version {
                DepositAddressInfoVersion::V0 => Some(self.address.clone()),
                _ => None,
            },
            eth_address_and_nonce_hash: match self.version {
                DepositAddressInfoVersion::V0 => Some(hash_string.clone()),
                _ => None,
            },
            address_and_nonce_hash: match self.version {
                DepositAddressInfoVersion::V0 => None,
                _ => Some(hash_string),
            },
            chain_id_hex: match self.version {
                DepositAddressInfoVersion::V0 | DepositAddressInfoVersion::V1 => None,
                _ => Some(hex::encode(&self.chain_id)),
            },
        }
    }

    pub fn validate(&self, btc_pub_key: &BtcPubKeySlice, network: &BtcNetwork) -> Result<()> {
        self.validate_commitment_hash()
            .and_then(|_| self.validate_btc_deposit_address(btc_pub_key, network))
            .and_then(|_| self.maybe_validate_chain_id())
    }

    fn maybe_validate_chain_id(&self) -> Result<()> {
        match self.version {
            DepositAddressInfoVersion::V0 | DepositAddressInfoVersion::V1 => {
                info!("✘ No need to check chain ID for version 0 or 1 deposit addresses!");
                Ok(())
            },
            DepositAddressInfoVersion::V2 | DepositAddressInfoVersion::V3 => {
                info!("✔ Validating chain ID in deposit address info version 2...");
                Self::validate_chain_id(&self.chain_id)
            },
        }
    }

    fn validate_chain_id(chain_id_bytes: &[Byte]) -> Result<()> {
        MetadataChainId::from_bytes(chain_id_bytes)
            .map(|chain_id| info!("✔ Chain ID successfully parsed: {}", chain_id))
    }

    #[cfg(test)]
    pub fn from_str(s: &str) -> Result<Self> {
        Self::from_json(&DepositAddressInfoJson::from_str(s)?)
    }

    fn calculate_btc_deposit_address(&self, pub_key: &BtcPubKeySlice, network: &BtcNetwork) -> BtcAddress {
        match self.version {
            DepositAddressInfoVersion::V0 => self.calculate_btc_deposit_address_v0(pub_key, network),
            DepositAddressInfoVersion::V1 => self.calculate_btc_deposit_address_v1(pub_key, network),
            DepositAddressInfoVersion::V2 => self.calculate_btc_deposit_address_v2(pub_key, network),
            DepositAddressInfoVersion::V3 => self.calculate_btc_deposit_address_v3(pub_key, network),
        }
    }

    fn calculate_btc_deposit_address_v0(&self, pub_key: &BtcPubKeySlice, network: &BtcNetwork) -> BtcAddress {
        let btc_script = get_p2sh_redeem_script_sig(&pub_key[..], &self.commitment_hash);
        BtcAddress::p2sh(&btc_script, *network)
    }

    fn calculate_btc_deposit_address_v1(&self, pub_key: &BtcPubKeySlice, network: &BtcNetwork) -> BtcAddress {
        self.calculate_btc_deposit_address_v0(pub_key, network)
    }

    fn calculate_btc_deposit_address_v2(&self, pub_key: &BtcPubKeySlice, network: &BtcNetwork) -> BtcAddress {
        self.calculate_btc_deposit_address_v0(pub_key, network)
    }

    fn calculate_btc_deposit_address_v3(&self, pub_key: &BtcPubKeySlice, network: &BtcNetwork) -> BtcAddress {
        self.calculate_btc_deposit_address_v0(pub_key, network)
    }

    fn validate_btc_deposit_address(&self, pub_key: &BtcPubKeySlice, network: &BtcNetwork) -> Result<()> {
        let calculated_address = self.calculate_btc_deposit_address(pub_key, network);
        if calculated_address != self.btc_deposit_address {
            debug!("   BTC deposit address: {}", self.btc_deposit_address);
            debug!("Calculated BTC address: {}", calculated_address);
            return Err("✘ Deposit info error - BTC deposit address is not valid!".into());
        }
        Ok(())
    }
}

pub fn validate_deposit_address_list_in_state<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    state
        .get_deposit_info_list()
        .and_then(|deposit_info_list| {
            deposit_info_list.validate(
                &state.btc_db_utils.get_btc_public_key_slice_from_db()?,
                &state.btc_db_utils.get_btc_network_from_db()?,
            )
        })
        .and(Ok(state))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AppError;

    fn get_sample_testnet_pub_key_hex() -> String {
        "03d2a5e3b162eb580fe2ce023cd5e0dddbb6286923acde77e3e5468314dc9373f7".to_string()
    }

    fn get_sample_mainnet_pub_key_hex() -> String {
        "0367663eeb293b978b495c20dee62cbfba551bf7e05a8381b374af84861ab6de39".to_string()
    }

    fn get_sample_btc_address() -> BtcAddress {
        BtcAddress::from_str("1DSh7vX6ed2cgTeKPwufV5i4hSi4pp373h").unwrap()
    }

    fn get_sample_testnet_deposit_info_json_string_v0() -> String {
        format!(
            "{{\"btc_deposit_address\":\"2N2LHYbt8K1KDBogd6XUG9VBv5YM6xefdM2\",\"eth_address\":\"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac\",\"eth_address_and_nonce_hash\":\"0x98eaf3812c998a46e0ee997ccdadf736c7bc13c18a5292df7a8d39089fd28d9e\",\"nonce\":1337,\"public_key\":\"{}\",\"version\":\"0\"}}",
            get_sample_testnet_pub_key_hex(),
        )
    }

    fn get_sample_testnet_deposit_info_json_string_v1() -> String {
        format!(
            "{{\"address\":\"0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC\",\"address_and_nonce_hash\":\"0x5364a60af6f1e0e8a0b0e38b8812e3c02b98727247d749500ee1e90066aa360e\",\"btc_deposit_address\":\"2NEqdGbbaHdCUBbSHRBgFVPNjgw3Gnt1zm5\",\"nonce\":1337,\"public_key\":\"{}\",\"version\":\"1\"}}",
            get_sample_testnet_pub_key_hex(),
        )
    }

    fn get_sample_testnet_deposit_info_json_string_v2() -> String {
        format!(
            "{{\"address\":\"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac\",\"address_and_nonce_hash\":\"0xecb9b83f750c30c63d28cac04203d30d863a2f5adc09148993d1072ca9bba96b\",\"btc_deposit_address\":\"2Mw2wsAcVuPMWD6vxSuEXc2HdUrpxaakYs6\",\"chain_id\":\"EthereumMainnet\",\"chain_id_hex\":\"0x005fe7f9\",\"nonce\":1633690512,\"public_key\":\"{}\",\"tool_version\":\"1.8.0\",\"user_data\":\"0xd3caffc0ff33\",\"version\":\"2\"}}",
            get_sample_testnet_pub_key_hex(),
        )
    }

    fn get_sample_mainnet_deposit_info_json_string_v0() -> String {
        format!(
            "{{\"btc_deposit_address\":\"3QtLZUeyy45utKbZnCt6tWFUoUQJ3vaME6\",\"eth_address\":\"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac\",\"eth_address_and_nonce_hash\":\"0x98eaf3812c998a46e0ee997ccdadf736c7bc13c18a5292df7a8d39089fd28d9e\",\"nonce\":1337,\"public_key\":\"{}\",\"tool_version\":\"1.6.0\",\"version\":\"0\"}}",
            get_sample_mainnet_pub_key_hex(),
        )
    }

    fn get_sample_mainnet_deposit_info_json_string_v1() -> String {
        format!(
            "{{\"address\":\"0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC\",\"address_and_nonce_hash\":\"0x5364a60af6f1e0e8a0b0e38b8812e3c02b98727247d749500ee1e90066aa360e\",\"btc_deposit_address\":\"3JQSxdUeiS6ss8UvAdwR52rzQiRnruuS4G\",\"nonce\":1337,\"public_key\":\"{}\",\"tool_version\":\"1.6.0\",\"version\":\"1\"}}",
            get_sample_mainnet_pub_key_hex(),
        )
    }

    fn get_sample_mainnet_deposit_info_json_string_v2() -> String {
        format!(
            "{{\"address\":\"0xfedfe2616eb3661cb8fed2782f5f0cc91d59dcac\",\"address_and_nonce_hash\":\"0xf8270f5b9b8b0b1434d73e8c7541bc73969dec9801ec8b72f9d205c989a458b3\",\"btc_deposit_address\":\"3H8hzBvP4jEwxFToAZhaKikVBr7zADj8oe\",\"chain_id\":\"EthereumMainnet\",\"chain_id_hex\":\"0x005fe7f9\",\"nonce\":1633690453,\"public_key\":\"{}\",\"tool_version\":\"1.8.0\",\"user_data\":\"0xd3caffc0ff33\",\"version\":\"2\"}}",
            get_sample_mainnet_pub_key_hex(),
        )
    }

    fn get_sample_testnet_deposit_info_v0() -> DepositAddressInfo {
        DepositAddressInfo::from_str(&get_sample_testnet_deposit_info_json_string_v0()).unwrap()
    }

    fn get_sample_testnet_deposit_info_v1() -> DepositAddressInfo {
        DepositAddressInfo::from_str(&get_sample_testnet_deposit_info_json_string_v1()).unwrap()
    }

    fn get_sample_testnet_deposit_info_v2() -> DepositAddressInfo {
        DepositAddressInfo::from_str(&get_sample_testnet_deposit_info_json_string_v2()).unwrap()
    }

    fn get_sample_mainnet_deposit_info_v0() -> DepositAddressInfo {
        DepositAddressInfo::from_str(&get_sample_mainnet_deposit_info_json_string_v0()).unwrap()
    }

    fn get_sample_mainnet_deposit_info_v1() -> DepositAddressInfo {
        DepositAddressInfo::from_str(&get_sample_mainnet_deposit_info_json_string_v1()).unwrap()
    }

    fn get_sample_mainnet_deposit_info_v2() -> DepositAddressInfo {
        DepositAddressInfo::from_str(&get_sample_mainnet_deposit_info_json_string_v2()).unwrap()
    }

    fn get_sample_btc_testnet_network() -> BtcNetwork {
        BtcNetwork::Testnet
    }

    fn get_sample_btc_mainnet_network() -> BtcNetwork {
        BtcNetwork::Bitcoin
    }

    fn get_sample_pub_key_slice(hex: &str) -> BtcPubKeySlice {
        let bytes = hex::decode(hex).unwrap();
        let mut arr = [0u8; 33];
        arr.copy_from_slice(&bytes);
        arr
    }

    fn get_sample_testnet_pub_key_slice() -> BtcPubKeySlice {
        get_sample_pub_key_slice(&get_sample_testnet_pub_key_hex())
    }

    fn get_sample_mainnet_pub_key_slice() -> BtcPubKeySlice {
        get_sample_pub_key_slice(&get_sample_mainnet_pub_key_hex())
    }

    fn get_sample_testnet_deposit_info_list() -> DepositInfoList {
        DepositInfoList::new(vec![
            get_sample_testnet_deposit_info_v0(),
            get_sample_testnet_deposit_info_v1(),
            get_sample_testnet_deposit_info_v2(),
        ])
    }

    fn get_sample_mainnet_deposit_info_list() -> DepositInfoList {
        DepositInfoList::new(vec![
            get_sample_mainnet_deposit_info_v0(),
            get_sample_mainnet_deposit_info_v1(),
            get_sample_mainnet_deposit_info_v2(),
        ])
    }

    fn get_sample_invalid_commitment_hash_testnet_deposit_info_list() -> DepositInfoList {
        DepositInfoList::new(
            get_sample_testnet_deposit_info_list()
                .iter()
                .cloned()
                .map(invalidate_commitment_hash)
                .collect(),
        )
    }

    fn get_sample_invalid_commitment_hash_mainnet_list() -> DepositInfoList {
        DepositInfoList::new(
            get_sample_mainnet_deposit_info_list()
                .iter()
                .cloned()
                .map(invalidate_commitment_hash)
                .collect(),
        )
    }

    fn get_sample_invalid_btc_address_testnet_deposit_info_list() -> DepositInfoList {
        DepositInfoList::new(
            get_sample_testnet_deposit_info_list()
                .iter()
                .cloned()
                .map(invalidate_btc_address)
                .collect(),
        )
    }

    fn get_sample_invalid_btc_address_mainnet_deposit_info_list() -> DepositInfoList {
        DepositInfoList::new(
            get_sample_mainnet_deposit_info_list()
                .iter()
                .cloned()
                .map(invalidate_btc_address)
                .collect(),
        )
    }

    fn invalidate_commitment_hash(mut deposit_info: DepositAddressInfo) -> DepositAddressInfo {
        deposit_info.nonce += 1;
        deposit_info
    }

    fn invalidate_btc_address(mut deposit_info: DepositAddressInfo) -> DepositAddressInfo {
        deposit_info.btc_deposit_address = get_sample_btc_address();
        deposit_info
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
        let user_data = None;
        let version = Some("1".to_string());
        let chain_id_hex = None;
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            user_data,
            eth_address,
            chain_id_hex,
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
        let user_data = None;
        let eth_address = None;
        let address_and_nonce_hash = None;
        let version = Some("0".to_string());
        let chain_id_hex = None;
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            user_data,
            eth_address,
            chain_id_hex,
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
        let user_data = None;
        let version = Some("1".to_string());
        let eth_address_and_nonce_hash = None;
        let chain_id_hex = None;
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            user_data,
            eth_address,
            chain_id_hex,
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
        let user_data = None;
        let chain_id_hex = None;
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            user_data,
            eth_address,
            chain_id_hex,
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
        let user_data = None;
        let address_and_nonce_hash = None;
        let chain_id_hex = None;
        let deposit_json = DepositAddressInfoJson {
            nonce,
            address,
            version,
            user_data,
            eth_address,
            chain_id_hex,
            btc_deposit_address,
            address_and_nonce_hash,
            eth_address_and_nonce_hash,
        };
        let result = DepositAddressInfo::from_json(&deposit_json).unwrap();
        assert_eq!(result.version, DepositAddressInfoVersion::V0);
    }

    #[test]
    fn should_convert_v0_testnet_deposit_info_string_to_deposit_info() {
        let json_str = get_sample_testnet_deposit_info_json_string_v0();
        let result = DepositAddressInfo::from_str(&json_str);
        assert!(result.is_ok());
    }

    #[test]
    fn should_convert_v1_testnet_deposit_info_string_to_deposit_info() {
        let json_str = get_sample_testnet_deposit_info_json_string_v1();
        let result = DepositAddressInfo::from_str(&json_str);
        assert!(result.is_ok());
    }

    #[test]
    fn should_convert_v2_testnet_deposit_info_string_to_deposit_info() {
        let json_str = get_sample_testnet_deposit_info_json_string_v2();
        let result = DepositAddressInfo::from_str(&json_str);
        assert!(result.is_ok())
    }

    #[test]
    fn testnet_deposit_info_list_should_be_valid() {
        let list = get_sample_testnet_deposit_info_list();
        let network = get_sample_btc_testnet_network();
        let pub_key = get_sample_testnet_pub_key_slice();
        let result = list.validate(&pub_key, &network);
        assert!(result.is_ok())
    }

    #[test]
    fn mainnet_deposit_info_list_should_be_valid() {
        let list = get_sample_mainnet_deposit_info_list();
        let network = get_sample_btc_mainnet_network();
        let pub_key = get_sample_mainnet_pub_key_slice();
        let result = list.validate(&pub_key, &network);
        assert!(result.is_ok())
    }

    #[test]
    fn invalid_commitment_hash_testnet_deposit_info_should_fail_validation() {
        let expected_err = "✘ Deposit info error - commitment hash is not valid!".to_string();
        let pub_key_slice = get_sample_testnet_pub_key_slice();
        let network = get_sample_btc_testnet_network();
        let invalid_list = get_sample_invalid_commitment_hash_testnet_deposit_info_list();
        invalid_list
            .iter()
            .for_each(|invalid_info| match invalid_info.validate(&pub_key_slice, &network) {
                Ok(_) => panic!("Should not be valid!"),
                Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
                Err(_) => panic!("Wrong error received!"),
            });
    }

    #[test]
    fn invalid_commitment_hash_mainnet_deposit_info_should_fail_validation() {
        let expected_err = "✘ Deposit info error - commitment hash is not valid!".to_string();
        let pub_key_slice = get_sample_mainnet_pub_key_slice();
        let network = get_sample_btc_mainnet_network();
        let invalid_list = get_sample_invalid_commitment_hash_mainnet_list();
        invalid_list
            .iter()
            .for_each(|invalid_info| match invalid_info.validate(&pub_key_slice, &network) {
                Ok(_) => panic!("Should not be valid!"),
                Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
                Err(_) => panic!("Wrong error received!"),
            });
    }

    #[test]
    fn invalid_btc_address_testnet_deposit_info_should_fail_validation() {
        let expected_err = "✘ Deposit info error - BTC deposit address is not valid!".to_string();
        let pub_key_slice = get_sample_testnet_pub_key_slice();
        let network = get_sample_btc_testnet_network();
        let invalid_list = get_sample_invalid_btc_address_testnet_deposit_info_list();
        invalid_list
            .iter()
            .for_each(|invalid_info| match invalid_info.validate(&pub_key_slice, &network) {
                Ok(_) => panic!("Should not be valid!"),
                Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
                Err(_) => panic!("Wrong error received!"),
            });
    }

    #[test]
    fn invalid_btc_address_hash_mainnet_deposit_info_should_fail_validation() {
        let expected_err = "✘ Deposit info error - BTC deposit address is not valid!".to_string();
        let pub_key_slice = get_sample_mainnet_pub_key_slice();
        let network = get_sample_btc_mainnet_network();
        let invalid_list = get_sample_invalid_btc_address_mainnet_deposit_info_list();
        invalid_list
            .iter()
            .for_each(|invalid_info| match invalid_info.validate(&pub_key_slice, &network) {
                Ok(_) => panic!("Should not be valid!"),
                Err(AppError::Custom(err)) => assert_eq!(err, expected_err),
                Err(_) => panic!("Wrong error received!"),
            });
    }

    #[test]
    fn should_validate_chain_id() {
        let valid_chain_id_bytes = MetadataChainId::EthereumMainnet.to_bytes().unwrap();
        let result = DepositAddressInfo::validate_chain_id(&valid_chain_id_bytes);
        assert!(result.is_ok());
    }

    #[test]
    fn should_fail_to_validate_invalid_chain_id() {
        let invalid_chain_id_bytes = hex::decode("d3adb33f").unwrap();
        assert!(MetadataChainId::from_bytes(&invalid_chain_id_bytes).is_err());
        let expected_error = format!(
            "Unrecognized bytes for `MetadataChainId`: 0x{}",
            hex::encode(&invalid_chain_id_bytes)
        );
        match DepositAddressInfo::validate_chain_id(&invalid_chain_id_bytes) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(error) => panic!("Wrong error! Got {}, expected {}", error, expected_error),
        };
    }

    #[test]
    fn should_validate_v3_deposit_address_info() {
        let info = DepositAddressInfo::from_str("{\"address\":\"someaddress\",\"address_and_nonce_hash\":\"0xe1da00e59d2d3d5fc5b3b76d0d087bb74d2ffe32dbb90bbb06c5146b40933cd0\",\"btc_deposit_address\":\"2N4sEHFhdDmAg9hn6ztVptcDpE5qWJB9fWv\",\"chain_id\":\"EthereumRopsten\",\"chain_id_hex\":\"0x0069c322\",\"nonce\":1645106870,\"public_key\":\"02cfae40b56f0706b059c48c4d2f22411f3c6f9f2e674bd5d764e93ab89d6f2efc\",\"tool_version\":\"1.9.0\",\"user_data\":\"0xc0ffee\",\"version\":\"3\"}").unwrap();
        let result = info.validate_commitment_hash();
        assert!(result.is_ok());
    }
}
