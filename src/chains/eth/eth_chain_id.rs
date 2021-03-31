use std::fmt;

use strum_macros::EnumIter;

use crate::{
    metadata::metadata_chain_id::MetadataChainId,
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
}

impl ChainId for EthChainId {}

impl EthChainId {
    fn to_bytes(&self) -> Result<Bytes> {
        Ok(self.to_u8().to_le_bytes().to_vec())
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        info!("✔ Getting `EthChainId` from bytes: {}", hex::encode(bytes));
        match convert_bytes_to_u8(bytes)? {
            1 => Ok(Self::Mainnet),
            3 => Ok(Self::Ropsten),
            4 => Ok(Self::Rinkeby),
            56 => Ok(Self::BscMainnet),
            _ => Err(format!("`EthChainId` error! Unrecognised bytes : {}", hex::encode(bytes)).into()),
        }
    }

    fn to_metadata_chain_id(&self) -> Result<MetadataChainId> {
        match self {
            Self::Mainnet => Ok(MetadataChainId::EthereumMainnet),
            Self::Rinkeby => Ok(MetadataChainId::EthereumRinkeby),
            Self::Ropsten => Ok(MetadataChainId::EthereumRopsten),
            Self::BscMainnet => Ok(MetadataChainId::BscMainnet),
        }
    }

    fn from_u8(chain_id: u8) -> Result<Self> {
        match chain_id {
            1 => Ok(Self::Mainnet),
            3 => Ok(Self::Ropsten),
            4 => Ok(Self::Rinkeby),
            56 => Ok(Self::BscMainnet),
            _ => Err(format!("`EthChainId` error! Unrecognized chain id: {}", chain_id).into()),
        }
    }

    fn to_u8(&self) -> u8 {
        match self {
            Self::Mainnet => 1,
            Self::Ropsten => 3,
            Self::Rinkeby => 4,
            Self::BscMainnet => 56,
        }
    }

    #[cfg(test)]
    fn get_all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter().collect()
    }
}

impl fmt::Display for EthChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Mainnet => write!(f, "ETH Mainnet: {}", self.to_u8()),
            Self::Rinkeby => write!(f, "Rinekby Testnet: {}", self.to_u8()),
            Self::Ropsten => write!(f, "Ropsten Testnet: {}", self.to_u8()),
            Self::BscMainnet => write!(f, "BSC Mainnet: {}", self.to_u8()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_u8_roundtrip_for_all_eth_chain_ids() {
        let ids = EthChainId::get_all();
        let u8s = ids.iter().map(|id| id.to_u8()).collect::<Vec<u8>>();
        let result = u8s
            .iter()
            .map(|u_8| EthChainId::from_u8(*u_8))
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
}
