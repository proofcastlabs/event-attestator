use std::fmt;

use bitcoin::network::constants::Network as BtcNetwork;
use ethereum_types::H256 as KeccakHash;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    crypto_utils::keccak_hash_bytes,
    metadata::metadata_chain_id::MetadataChainId,
    traits::ChainId,
    types::{Byte, Bytes, Result},
    utils::{convert_bytes_to_u64, convert_u64_to_bytes},
};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum BtcChainId {
    Bitcoin,
    Testnet,
}

impl ChainId for BtcChainId {
    fn keccak_hash(&self) -> Result<KeccakHash> {
        Ok(keccak_hash_bytes(&self.to_bytes()))
    }
}

impl BtcChainId {
    fn to_btc_network(&self) -> BtcNetwork {
        match self {
            Self::Bitcoin => BtcNetwork::Bitcoin,
            Self::Testnet => BtcNetwork::Testnet,
        }
    }

    fn to_metadata_chain_id(&self) -> Result<MetadataChainId> {
        match self {
            Self::Bitcoin => Ok(MetadataChainId::BitcoinMainnet),
            Self::Testnet => Ok(MetadataChainId::BitcoinTestnet),
        }
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        match convert_bytes_to_u64(bytes)? {
            0 => Ok(Self::Bitcoin),
            1 => Ok(Self::Testnet),
            _ => Err(format!("`BtcChainId` error! Unrecognised byte: 0x{}", hex::encode(bytes)).into()),
        }
    }

    fn to_bytes(&self) -> Bytes {
        match self {
            Self::Bitcoin => convert_u64_to_bytes(0),
            Self::Testnet => convert_u64_to_bytes(1),
        }
    }

    fn to_hex(&self) -> String {
        hex::encode(&self.to_bytes())
    }

    fn get_all() -> Vec<Self> {
        Self::iter().collect()
    }
}

impl fmt::Display for BtcChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bitcoin => write!(f, "Bitcoin Mainnet: 0x{}", self.to_hex()),
            Self::Testnet => write!(f, "Bitcoin Testnet: 0x{}", self.to_hex()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_bytes_roundtrip_for_all_btc_chain_ids() {
        let ids = BtcChainId::get_all();
        let vec_of_bytes = ids.iter().map(|id| id.to_bytes()).collect::<Vec<Bytes>>();
        let result = vec_of_bytes
            .iter()
            .map(|ref bytes| BtcChainId::from_bytes(bytes))
            .collect::<Result<Vec<BtcChainId>>>()
            .unwrap();
        assert_eq!(result, ids);
    }
}
