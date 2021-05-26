use std::fmt;

use bitcoin::network::constants::Network as BtcNetwork;
use ethereum_types::H256 as KeccakHash;
use strum_macros::EnumIter;

use crate::{
    crypto_utils::keccak_hash_bytes,
    traits::ChainId,
    types::{Byte, Bytes, Result},
    utils::{convert_bytes_to_u64, convert_u64_to_bytes},
};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum BtcChainId {
    Bitcoin,
    Testnet,
    Unknown(Bytes),
}

impl ChainId for BtcChainId {
    fn keccak_hash(&self) -> Result<KeccakHash> {
        Ok(keccak_hash_bytes(match self {
            Self::Bitcoin => "Bitcoin".as_bytes(),
            Self::Testnet => "Testnet".as_bytes(),
            Self::Unknown(bytes) => bytes,
        }))
    }
}

impl BtcChainId {
    pub fn to_btc_network(&self) -> BtcNetwork {
        match self {
            Self::Testnet => BtcNetwork::Testnet,
            _ => BtcNetwork::Bitcoin,
        }
    }

    pub fn from_btc_network(btc_network: &BtcNetwork) -> Result<Self> {
        match btc_network {
            BtcNetwork::Bitcoin => Ok(Self::Bitcoin),
            BtcNetwork::Testnet => Ok(Self::Testnet),
            _ => Err(format!("`BtcChainId` error! Unsupported BtcNetwork: {}", btc_network).into()),
        }
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        match convert_bytes_to_u64(bytes)? {
            0 => Ok(Self::Bitcoin),
            1 => Ok(Self::Testnet),
            _ => Ok(Self::Unknown(bytes.to_vec())),
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        match self {
            Self::Bitcoin => convert_u64_to_bytes(0),
            Self::Testnet => convert_u64_to_bytes(1),
            Self::Unknown(bytes) => bytes.to_vec(),
        }
    }

    fn to_hex(&self) -> String {
        hex::encode(&self.to_bytes())
    }

    #[cfg(test)]
    fn get_all() -> Vec<Self> {
        use strum::IntoEnumIterator;
        Self::iter()
            .filter(|x| match x {
                Self::Unknown(_) => false,
                _ => true,
            })
            .collect()
    }
}

impl fmt::Display for BtcChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Bitcoin => write!(f, "Bitcoin Mainnet: 0x{}", self.to_hex()),
            Self::Testnet => write!(f, "Bitcoin Testnet: 0x{}", self.to_hex()),
            Self::Unknown(_) => write!(f, "Bitcoin Testnet: 0x{}", self.to_hex()),
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
