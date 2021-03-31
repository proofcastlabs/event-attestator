use std::fmt;

use ethereum_types::H256 as KeccakHash;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    chains::{btc::btc_chain_id::BtcChainId, eos::eos_chain_id::EosChainId, eth::eth_chain_id::EthChainId},
    metadata::metadata_protocol_id::MetadataProtocolId,
    traits::ChainId,
    types::{Byte, Bytes, Result},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, EnumIter)]
pub enum MetadataChainId {
    EthereumMainnet,  // 0x005fe7f9
    EthereumRopsten,  // 0x0069c322
    EthereumRinkeby,  // 0x00f34368
    BitcoinMainnet,   // 0x01ec97de
    BitcoinTestnet,   // 0x018afeb2
    EosMainnet,       // 0x02e7261c
    TelosMainnet,     // 0x028c7109
    BscMainnet,       // 0x00e4b170
    EosJungleTestnet, // 0x0282317f
}

impl MetadataChainId {
    fn get_all() -> Vec<Self> {
        Self::iter().collect()
    }

    pub fn to_protocol_id(&self) -> MetadataProtocolId {
        match self {
            Self::EosMainnet | Self::TelosMainnet | Self::EosJungleTestnet => MetadataProtocolId::Eos,
            Self::BitcoinMainnet | Self::BitcoinTestnet => MetadataProtocolId::Bitcoin,
            Self::EthereumMainnet | Self::EthereumRinkeby | Self::EthereumRopsten | Self::BscMainnet => {
                MetadataProtocolId::Ethereum
            },
        }
    }

    fn to_chain_id(&self) -> Box<dyn ChainId> {
        match self {
            Self::EthereumMainnet => Box::new(EthChainId::Mainnet),
            Self::EthereumRinkeby => Box::new(EthChainId::Rinkeby),
            Self::EthereumRopsten => Box::new(EthChainId::Ropsten),
            Self::BscMainnet => Box::new(EthChainId::BscMainnet),
            Self::BitcoinMainnet => Box::new(BtcChainId::Bitcoin),
            Self::BitcoinTestnet => Box::new(BtcChainId::Testnet),
            Self::EosMainnet => Box::new(EosChainId::EosMainnet),
            Self::TelosMainnet => Box::new(EosChainId::TelosMainnet),
            Self::EosJungleTestnet => Box::new(EosChainId::EosJungleTestnet),
        }
    }

    fn to_hex(&self) -> Result<String> {
        Ok(hex::encode(self.to_bytes()?))
    }

    fn to_keccak_hash(&self) -> Result<KeccakHash> {
        self.to_chain_id().keccak_hash()
    }

    fn to_first_three_bytes_of_keccak_hash(self) -> Result<Bytes> {
        Ok(self.to_keccak_hash()?[..3].to_vec())
    }

    pub fn to_bytes(&self) -> Result<Bytes> {
        Ok(vec![
            vec![self.to_protocol_id().to_byte()],
            self.to_first_three_bytes_of_keccak_hash()?,
        ]
        .concat())
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        let maybe_self = Self::get_all()
            .iter()
            .map(|id| match id.to_bytes() {
                Err(_) => None,
                Ok(id_bytes) => {
                    if id_bytes == bytes {
                        Some(id.clone())
                    } else {
                        None
                    }
                },
            })
            .filter(Option::is_some)
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
        let err_msg = "Could not unwrap hex!".to_string();
        match self {
            Self::EosMainnet => write!(f, "Eos Mainnet: 0x{}", self.to_hex().unwrap_or_else(|_| err_msg)),
            Self::TelosMainnet => write!(f, "Telos Mainnet: 0x{}", self.to_hex().unwrap_or_else(|_| err_msg)),
            Self::BitcoinMainnet => write!(f, "Bitcoin Mainnet: 0x{}", self.to_hex().unwrap_or_else(|_| err_msg)),
            Self::BitcoinTestnet => write!(f, "Bitcoin Testnet: 0x{}", self.to_hex().unwrap_or_else(|_| err_msg)),
            Self::EthereumMainnet => write!(f, "Ethereum Mainnet: 0x{}", self.to_hex().unwrap_or_else(|_| err_msg)),
            Self::EthereumRinkeby => write!(f, "Ethereum Rinkeby: 0x{}", self.to_hex().unwrap_or_else(|_| err_msg)),
            Self::EthereumRopsten => write!(f, "Ethereum Ropsten: 0x{}", self.to_hex().unwrap_or_else(|_| err_msg)),
            Self::EosJungleTestnet => write!(f, "EOS Jungle Testnet: 0x{}", self.to_hex().unwrap_or_else(|_| err_msg)),
            Self::BscMainnet => write!(
                f,
                "Binance Chain (BSC) Mainnet: 0x{}",
                self.to_hex().unwrap_or_else(|_| err_msg)
            ),
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
            let byte = id.to_bytes().unwrap();
            let result = MetadataChainId::from_bytes(&byte).unwrap();
            assert_eq!(&result, id);
        });
    }

    #[test]
    fn all_chain_ids_should_be_unique() {
        let mut ids_as_bytes = MetadataChainId::get_all()
            .iter()
            .map(|id| id.to_bytes().unwrap())
            .collect::<Vec<Bytes>>();
        ids_as_bytes.sort();
        let length_before_dedup = ids_as_bytes.len();
        ids_as_bytes.dedup();
        let length_after_dedup = ids_as_bytes.len();
        assert_eq!(length_before_dedup, length_after_dedup);
    }

    #[test]
    fn should_get_metadata_chain_id_from_bytes_correctly() {
        let chain_ids_bytes = vec![
            "005fe7f9", "0069c322", "00f34368", "01ec97de", "018afeb2", "02e7261c", "028c7109", "00e4b170", "0282317f",
        ]
        .iter()
        .map(|ref hex| hex::decode(hex).unwrap())
        .collect::<Vec<Bytes>>();
        let result = chain_ids_bytes
            .iter()
            .map(|ref bytes| MetadataChainId::from_bytes(bytes))
            .collect::<Result<Vec<MetadataChainId>>>()
            .unwrap();
        assert_eq!(result, MetadataChainId::get_all());
    }
}
