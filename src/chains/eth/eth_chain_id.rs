use std::fmt;

use ethereum_types::H256 as KeccakHash;
use strum_macros::EnumIter;

use crate::{
    crypto_utils::keccak_hash_bytes,
    errors::AppError,
    metadata::{metadata_chain_id::MetadataChainId, metadata_traits::ToMetadataChainId},
    traits::ChainId,
    types::{Byte, Bytes, Result},
    utils::convert_bytes_to_u8,
};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum EthChainId {
    Mainnet,
    Rinkeby,
    Ropsten,
    BscMainnet,
    XDaiMainnet,
    InterimChain,
    PolygonMainnet,
    Unknown(u8),
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
            "interim" | "255" => Ok(Self::InterimChain),
            "polygon" | "137" => Ok(Self::PolygonMainnet),
            _ => match s.parse::<u8>() {
                Ok(byte) => Ok(Self::Unknown(byte)),
                Err(_) => Err(format!("✘ Unrecognized ETH network: '{}'!", s).into()),
            },
        }
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(self.to_u8().to_le_bytes().to_vec())
    }

    pub fn to_byte(&self) -> Byte {
        match self {
            Self::Mainnet => 1,
            Self::Rinkeby => 4,
            Self::Ropsten => 3,
            Self::BscMainnet => 56,
            Self::XDaiMainnet => 100,
            Self::InterimChain => 255,
            Self::PolygonMainnet => 137,
            Self::Unknown(byte) => *byte,
        }
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        info!("✔ Getting `EthChainId` from bytes: {}", hex::encode(bytes));
        let byte = convert_bytes_to_u8(bytes)?;
        match byte {
            1 => Ok(Self::Mainnet),
            3 => Ok(Self::Ropsten),
            4 => Ok(Self::Rinkeby),
            56 => Ok(Self::BscMainnet),
            100 => Ok(Self::XDaiMainnet),
            255 => Ok(Self::InterimChain),
            137 => Ok(Self::PolygonMainnet),
            _ => {
                info!("✔ Using unknown ETH chain ID: 0x{}", hex::encode(bytes));
                Ok(Self::Unknown(byte))
            },
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
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            Self::Mainnet => 1,
            Self::Ropsten => 3,
            Self::Rinkeby => 4,
            Self::BscMainnet => 56,
            Self::XDaiMainnet => 100,
            Self::InterimChain => 255,
            Self::PolygonMainnet => 137,
            Self::Unknown(byte) => *byte,
        }
    }
}

#[cfg(test)]
impl EthChainId {
    fn is_unknown(&self) -> bool {
        match self {
            Self::Unknown(_) => true,
            _ => false,
        }
    }

    fn get_all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().filter(|chain_id| !chain_id.is_unknown()).collect()
    }

    fn to_hex(&self) -> Result<String> {
        self.to_bytes().map(|ref bytes| hex::encode(bytes))
    }

    fn to_keccak_hash_hex(&self) -> Result<String> {
        self.keccak_hash().map(|ref bytes| hex::encode(bytes))
    }
}

impl fmt::Display for EthChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Mainnet => write!(f, "ETH Mainnet: {}", self.to_u8()),
            Self::Rinkeby => write!(f, "Rinekby Testnet: {}", self.to_u8()),
            Self::Ropsten => write!(f, "Ropsten Testnet: {}", self.to_u8()),
            Self::BscMainnet => write!(f, "BSC Mainnet: {}", self.to_u8()),
            Self::XDaiMainnet => write!(f, "xDai Mainnet: {}", self.to_u8()),
            Self::PolygonMainnet => write!(f, "Polygon Mainnet: {}", self.to_u8()),
            Self::InterimChain => write!(f, "Interim Chain: {}", self.to_u8()),
            Self::Unknown(_) => write!(f, "Unkown ETH chain ID: {}", self.to_u8()),
        }
    }
}

impl TryFrom<u8> for EthChainId {
    type Error = AppError;

    fn try_from(byte: u8) -> Result<Self> {
        Self::from_bytes(&[byte])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_u8_roundtrip_for_all_eth_chain_ids() {
        let ids = EthChainId::get_all();
        let bytes = ids.iter().map(|id| id.to_u8()).collect::<Vec<u8>>();
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

    fn get_legacy_chain_ids() -> Vec<EthChainId> {
        vec![
            EthChainId::Mainnet,
            EthChainId::Rinkeby,
            EthChainId::Ropsten,
            EthChainId::BscMainnet,
            EthChainId::XDaiMainnet,
            EthChainId::InterimChain,
            EthChainId::PolygonMainnet,
        ]
    }

    fn get_legacy_chain_ids_hex<'a>() -> Vec<&'a str> {
        vec!["01", "04", "03", "38", "64", "ff", "89"]
    }

    fn get_legacy_chain_ids_keccak_hashes<'a>() -> Vec<&'a str> {
        vec![
            "5fe7f977e71dba2ea1a68e21057beebb9be2ac30c6410aa38d4f3fbe41dcffd2",
            "f343681465b9efe82c933c3e8748c70cb8aa06539c361de20f72eac04e766393",
            "69c322e3248a5dfc29d73c5b0553b0185a35cd5bb6386747517ef7e53b15e287",
            "e4b1702d9298fee62dfeccc57d322a463ad55ca201256d01f62b45b2e1c21c10",
            "f1918e8562236eb17adc8502332f4c9c82bc14e19bfc0aa10ab674ff75b3d2f3",
            "8b1a944cf13a9a1c08facb2c9e98623ef3254d2ddb48113885c3e8e97fec8db9",
            "75dd4ce35898634c43d8e291c5edc041d288f0c0a531e92d5528804add589d1f",
        ]
    }

    #[test]
    fn should_get_all_chain_id_legacy_bytes() {
        let legacy_chain_ids = get_legacy_chain_ids();
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
        let legacy_chain_ids = get_legacy_chain_ids();
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
