#![allow(clippy::redundant_closure)]
use std::fmt;

use ethereum_types::H256 as EthHash;

use crate::{
    chains::eth::eth_crypto_utils::keccak_hash_bytes,
    metadata::metadata_protocol_id::MetadataProtocolId,
    types::{Byte, Bytes, Result},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataChainId {
    EthereumMainnet, // 0x005fe7f9
    EthereumRinkeby, // 0x0069c322
    EthereumRopsten, // 0x00f34368
    BitcoinMainnet,  // 0x01ec97de
    BitcoinTestnet,  // 0x018afeb2
    EosMainnet,      // 0x02e7261c
    TelosMainnet,    // 0x028c7109
    BscMainnet,      // 0x00e4b170
}

impl MetadataChainId {
    fn get_all() -> Vec<Self> {
        // TODO How to ensure this vec always contains all members?
        vec![
            Self::EthereumMainnet,
            Self::EthereumRopsten,
            Self::EthereumRinkeby,
            Self::BitcoinMainnet,
            Self::BitcoinTestnet,
            Self::EosMainnet,
            Self::TelosMainnet,
            Self::BscMainnet,
        ]
    }

    pub fn to_protocol_id(&self) -> MetadataProtocolId {
        match self {
            Self::EosMainnet | Self::TelosMainnet => MetadataProtocolId::Eos,
            Self::BitcoinMainnet | Self::BitcoinTestnet => MetadataProtocolId::Bitcoin,
            Self::EthereumMainnet | Self::EthereumRinkeby | Self::EthereumRopsten | Self::BscMainnet => {
                MetadataProtocolId::Ethereum
            },
        }
    }

    fn to_hash(&self) -> EthHash {
        keccak_hash_bytes(&match self {
            Self::EthereumMainnet => {
                let chain_id = 1u8;
                chain_id.to_le_bytes().to_vec()
            },
            Self::EthereumRinkeby => {
                let chain_id = 3u8;
                chain_id.to_le_bytes().to_vec()
            },
            Self::EthereumRopsten => {
                let chain_id = 4u8;
                chain_id.to_le_bytes().to_vec()
            },
            Self::BscMainnet => {
                let chain_id = 56u8;
                chain_id.to_le_bytes().to_vec()
            },
            Self::BitcoinMainnet => {
                let chain_id = "Bitcoin";
                chain_id.as_bytes().to_vec()
            },
            Self::BitcoinTestnet => {
                let chain_id = "Testnet";
                chain_id.as_bytes().to_vec()
            },
            Self::EosMainnet => {
                let chain_id = "aca376f206b8fc25a6ed44dbdc66547c36c6c33e3a119ffbeaef943642f0e906";
                hex::decode(chain_id).unwrap_or_else(|_| vec![])
            },
            Self::TelosMainnet => {
                let chain_id = "4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11";
                hex::decode(chain_id).unwrap_or_else(|_| vec![])
            },
        })
    }

    fn to_hex(&self) -> String {
        hex::encode(self.to_bytes())
    }

    pub fn to_bytes(&self) -> Bytes {
        vec![vec![self.to_protocol_id().to_byte()], self.to_hash()[..3].to_vec()].concat()
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        let maybe_self = Self::get_all()
            .iter()
            .map(|id| {
                let id_bytes = id.to_bytes();
                if id_bytes == bytes {
                    Some(id.clone())
                } else {
                    None
                }
            })
            .filter(|x| x.is_some())
            .collect::<Vec<Option<Self>>>();
        match maybe_self.len() {
            1 => maybe_self[0]
                .clone()
                .ok_or_else(|| "Failed to unwrap `maybe_self` from option!".into()),
            0 => Err(format!("Unrecognized bytes for `MetadataChainId`: 0x{}", hex::encode(bytes)).into()),
            _ => Err("`MetadataChainId` collision! > 1 chain ID has the same 1st 3 bytes when hashed!".into()),
        }
    }

    #[cfg(test)]
    fn print_all() {
        Self::get_all().iter().for_each(|id| println!("{}", id))
    }

    pub fn from_eth_chain_id(eth_chain_id: u8) -> Result<Self> {
        match eth_chain_id {
            1 => Ok(Self::EthereumMainnet),
            3 => Ok(Self::EthereumRinkeby),
            4 => Ok(Self::EthereumRopsten),
            56 => Ok(Self::BscMainnet),
            _ => Err(format!("Unsupported ETH chain ID: {}", eth_chain_id).into()),
        }
    }
}

impl fmt::Display for MetadataChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EosMainnet => write!(f, "Eos Mainnet: 0x{}", self.to_hex()),
            Self::TelosMainnet => write!(f, "Telos Mainnet: 0x{}", self.to_hex()),
            Self::BitcoinMainnet => write!(f, "Bitcoin Mainnet: 0x{}", self.to_hex()),
            Self::BitcoinTestnet => write!(f, "Bitcoin Testnet: 0x{}", self.to_hex()),
            Self::EthereumMainnet => write!(f, "Ethereum Mainnet: 0x{}", self.to_hex()),
            Self::EthereumRinkeby => write!(f, "Ethereum Rinkeby: 0x{}", self.to_hex()),
            Self::EthereumRopsten => write!(f, "Ethereum Ropsten: 0x{}", self.to_hex()),
            Self::BscMainnet => write!(f, "Binance Chain (BSC) Mainnet: 0x{}", self.to_hex()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_print_all_ids() {
        MetadataChainId::print_all();
    }

    #[test]
    fn should_perform_metadata_chain_ids_bytes_round_trip() {
        MetadataChainId::get_all().iter().for_each(|id| {
            let byte = id.to_bytes();
            let result = MetadataChainId::from_bytes(&byte).unwrap();
            assert_eq!(&result, id);
        });
    }

    #[test]
    fn all_chain_ids_should_be_unique() {
        let mut ids_as_bytes = MetadataChainId::get_all()
            .iter()
            .map(|id| id.to_bytes())
            .collect::<Vec<Bytes>>();
        ids_as_bytes.sort();
        let length_before_dedup = ids_as_bytes.len();
        ids_as_bytes.dedup();
        let length_after_dedup = ids_as_bytes.len();
        assert_eq!(length_before_dedup, length_after_dedup);
    }
}
