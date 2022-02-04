use std::fmt;

use ethereum_types::H256 as KeccakHash;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

use crate::{
    crypto_utils::keccak_hash_bytes,
    errors::AppError,
    metadata::{metadata_chain_id::MetadataChainId, metadata_traits::ToMetadataChainId},
    traits::ChainId,
    types::{Byte, Bytes, Result},
    utils::{convert_bytes_to_u64, convert_bytes_to_u8},
};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
pub enum EthChainId {
    Mainnet,
    Rinkeby,
    Ropsten,
    BscMainnet,
    XDaiMainnet,
    InterimChain,
    PolygonMainnet,
    ArbitrumMainnet,
    LuxochainMainnet,
    Unknown(u64),
}

impl Default for EthChainId {
    fn default() -> Self {
        Self::Mainnet
    }
}

impl ChainId for EthChainId {
    fn keccak_hash(&self) -> Result<KeccakHash> {
        Ok(keccak_hash_bytes(&self.to_bytes()?))
    }
}

impl ToMetadataChainId for EthChainId {
    fn to_metadata_chain_id(&self) -> MetadataChainId {
        match self {
            Self::Unknown(_) => MetadataChainId::EthUnknown,
            Self::BscMainnet => MetadataChainId::BscMainnet,
            Self::XDaiMainnet => MetadataChainId::XDaiMainnet,
            Self::Mainnet => MetadataChainId::EthereumMainnet,
            Self::Rinkeby => MetadataChainId::EthereumRinkeby,
            Self::Ropsten => MetadataChainId::EthereumRopsten,
            Self::InterimChain => MetadataChainId::InterimChain,
            Self::PolygonMainnet => MetadataChainId::PolygonMainnet,
            Self::ArbitrumMainnet => MetadataChainId::ArbitrumMainnet,
            Self::LuxochainMainnet => MetadataChainId::LuxochainMainnet,
        }
    }
}

impl EthChainId {
    pub fn unknown() -> Self {
        Self::Unknown(0)
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match &*s.to_lowercase() {
            "mainnet" | "1" => Ok(Self::Mainnet),
            "ropsten" | "3" => Ok(Self::Ropsten),
            "rinkeby" | "4" => Ok(Self::Rinkeby),
            "bsc" | "56" => Ok(Self::BscMainnet),
            "xdai" | "100" => Ok(Self::XDaiMainnet),
            "interim" | "947" => Ok(Self::InterimChain),
            "polygon" | "137" => Ok(Self::PolygonMainnet),
            "arbitrum" | "42161" => Ok(Self::ArbitrumMainnet),
            "luxo" | "luxochain" | "110" => Ok(Self::LuxochainMainnet),
            _ => match s.parse::<u64>() {
                Ok(u_64) => Ok(Self::Unknown(u_64)),
                Err(_) => Err(format!("✘ Unrecognized ETH network: '{}'!", s).into()),
            },
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        match self {
            // NOTE: The following explicit ones are for legacy reasons.
            Self::Mainnet => Ok(vec![0x01]),
            Self::Rinkeby => Ok(vec![0x04]),
            Self::Ropsten => Ok(vec![0x03]),
            Self::BscMainnet => Ok(vec![0x38]),
            Self::XDaiMainnet => Ok(vec![0x64]),
            Self::PolygonMainnet => Ok(vec![0x89]),
            _ => Ok(self.to_u64().to_le_bytes().to_vec()),
        }
    }

    fn from_unsigned_int<T: Into<u64>>(i: T) -> Result<Self> {
        let needle: u64 = i.into();
        match needle {
            1 => Ok(Self::Mainnet),
            3 => Ok(Self::Ropsten),
            4 => Ok(Self::Rinkeby),
            56 => Ok(Self::BscMainnet),
            100 => Ok(Self::XDaiMainnet),
            947 => Ok(Self::InterimChain),
            137 => Ok(Self::PolygonMainnet),
            110 => Ok(Self::LuxochainMainnet),
            42161 => Ok(Self::ArbitrumMainnet),
            _ => {
                info!("✔ Using unknown ETH chain ID: {}", needle);
                Ok(Self::Unknown(needle))
            },
        }
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        let hex = hex::encode(bytes);
        if bytes.len() == 1 {
            info!("✔ Getting `EthChainId` from legacy byte: 0x{}", hex);
            convert_bytes_to_u8(bytes).and_then(Self::from_unsigned_int)
        } else {
            info!("✔ Getting `EthChainId` from bytes: 0x{}", hex);
            convert_bytes_to_u64(bytes).and_then(Self::from_unsigned_int)
        }
    }

    pub fn to_metadata_chain_id(&self) -> MetadataChainId {
        match self {
            Self::BscMainnet => MetadataChainId::BscMainnet,
            Self::Unknown(_) => MetadataChainId::EthUnknown,
            Self::XDaiMainnet => MetadataChainId::XDaiMainnet,
            Self::Mainnet => MetadataChainId::EthereumMainnet,
            Self::Rinkeby => MetadataChainId::EthereumRinkeby,
            Self::Ropsten => MetadataChainId::EthereumRopsten,
            Self::InterimChain => MetadataChainId::InterimChain,
            Self::PolygonMainnet => MetadataChainId::PolygonMainnet,
            Self::ArbitrumMainnet => MetadataChainId::ArbitrumMainnet,
            Self::LuxochainMainnet => MetadataChainId::LuxochainMainnet,
        }
    }

    pub fn to_u64(&self) -> u64 {
        match self {
            Self::Mainnet => 1,
            Self::Ropsten => 3,
            Self::Rinkeby => 4,
            Self::BscMainnet => 56,
            Self::XDaiMainnet => 100,
            Self::InterimChain => 947,
            Self::PolygonMainnet => 137,
            Self::LuxochainMainnet => 110,
            Self::ArbitrumMainnet => 42161,
            Self::Unknown(u_64) => *u_64,
        }
    }
}

#[cfg(test)]
impl EthChainId {
    fn to_hex(&self) -> Result<String> {
        self.to_bytes().map(|ref bytes| hex::encode(bytes))
    }

    fn to_keccak_hash_hex(&self) -> Result<String> {
        self.keccak_hash().map(|ref bytes| hex::encode(bytes))
    }

    fn get_all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().filter(|chain_id| !chain_id.is_unknown()).collect()
    }

    fn is_unknown(&self) -> bool {
        match self {
            Self::Unknown(_) => true,
            _ => false,
        }
    }
}

impl fmt::Display for EthChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let u_64 = self.to_u64();
        match self {
            Self::Mainnet => write!(f, "ETH Mainnet: {}", u_64),
            Self::BscMainnet => write!(f, "BSC Mainnet: {}", u_64),
            Self::Rinkeby => write!(f, "Rinekby Testnet: {}", u_64),
            Self::Ropsten => write!(f, "Ropsten Testnet: {}", u_64),
            Self::XDaiMainnet => write!(f, "xDai Mainnet: {}", u_64),
            Self::InterimChain => write!(f, "Interim Chain: {}", u_64),
            Self::Unknown(_) => write!(f, "Unkown ETH chain ID: {}", u_64),
            Self::PolygonMainnet => write!(f, "Polygon Mainnet: {}", u_64),
            Self::ArbitrumMainnet => write!(f, "Abritrum Mainnet: {}", u_64),
            Self::LuxochainMainnet => write!(f, "Luxochain Mainnet: {}", u_64),
        }
    }
}

impl TryFrom<u64> for EthChainId {
    type Error = AppError;

    fn try_from(u_64: u64) -> Result<Self> {
        Self::from_bytes(&u_64.to_le_bytes())
    }
}

impl TryFrom<u8> for EthChainId {
    type Error = AppError;

    fn try_from(u_8: u8) -> Result<Self> {
        Self::try_from(u_8 as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_u64_roundtrip_for_all_eth_chain_ids() {
        let ids = EthChainId::get_all();
        let bytes = ids.iter().map(|id| id.to_u64()).collect::<Vec<u64>>();
        let result = bytes
            .iter()
            .map(|byte| EthChainId::try_from(*byte))
            .collect::<Result<Vec<EthChainId>>>()
            .unwrap();
        assert_eq!(result, ids);
    }

    #[test]
    fn should_make_bytes_roundtrip_for_all_eth_chain_ids() {
        let ids = EthChainId::get_all();
        let vec_of_bytes = ids
            .iter()
            .map(|id| id.to_bytes())
            .collect::<Result<Vec<Bytes>>>()
            .unwrap();
        let result = vec_of_bytes
            .iter()
            .map(|ref bytes| EthChainId::from_bytes(bytes))
            .collect::<Result<Vec<EthChainId>>>()
            .unwrap();
        assert_eq!(result, ids);
    }

    fn get_all_legacy() -> Vec<EthChainId> {
        vec![
            EthChainId::Mainnet,
            EthChainId::Rinkeby,
            EthChainId::Ropsten,
            EthChainId::BscMainnet,
            EthChainId::XDaiMainnet,
            EthChainId::PolygonMainnet,
        ]
    }

    fn get_legacy_chain_ids_hex<'a>() -> Vec<&'a str> {
        vec!["01", "04", "03", "38", "64", "89"]
    }

    fn get_legacy_chain_ids_keccak_hashes<'a>() -> Vec<&'a str> {
        vec![
            "5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2",
            "f343681465b9efe82c933c3e8748c70cb8aa06539c361de20f72eac04e766393",
            "69c322e3248a5dfc29d73c5b0553b0185a35cd5bb6386747517ef7e53b15e287",
            "e4b1702d9298fee62dfeccc57d322a463ad55ca201256d01f62b45b2e1c21c10",
            "f1918e8562236eb17adc8502332f4c9c82bc14e19bfc0aa10ab674ff75b3d2f3",
            "75dd4ce35898634c43d8e291c5edc041d288f0c0a531e92d5528804add589d1f",
        ]
    }

    #[test]
    fn should_get_all_chain_id_legacy_bytes() {
        let legacy_chain_ids = get_all_legacy();
        let chain_ids_hex = legacy_chain_ids
            .iter()
            .map(|id| id.to_hex())
            .collect::<Result<Vec<String>>>()
            .unwrap();
        let expected_chain_ids_hex = get_legacy_chain_ids_hex();
        chain_ids_hex
            .iter()
            .enumerate()
            .for_each(|(i, chain_id_hex)| assert_eq!(chain_id_hex, expected_chain_ids_hex[i]));
    }

    #[test]
    fn shuld_get_all_chain_id_legacy_keccak_hashes() {
        let legacy_chain_ids = get_all_legacy();
        let chain_ids_keccak_hashes = legacy_chain_ids
            .iter()
            .map(|id| id.to_keccak_hash_hex())
            .collect::<Result<Vec<String>>>()
            .unwrap();
        let expected_chain_ids_keccak_hashes = get_legacy_chain_ids_keccak_hashes();
        chain_ids_keccak_hashes
            .iter()
            .enumerate()
            .for_each(|(i, chain_id_hex)| assert_eq!(chain_id_hex, expected_chain_ids_keccak_hashes[i]));
    }
}
