#![allow(dead_code)] // FIXME rm!

use std::fmt;

use crate::types::{Byte, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockchainId {
    EthereumMainnet,
    EthereumRopsten,
    EthereumRinkeby,
    BitcoinMainnet,
    BitcoinTestnet,
    EosMainnet,
    TelosMainnet,
}

impl BlockchainId {
    pub fn as_byte(&self) -> Byte {
        match self {
            BlockchainId::EthereumMainnet => 0x00,
            BlockchainId::EthereumRinkeby => 0x01,
            BlockchainId::EthereumRopsten => 0x02,
            BlockchainId::BitcoinMainnet => 0x03,
            BlockchainId::BitcoinTestnet => 0x04,
            BlockchainId::EosMainnet => 0x05,
            BlockchainId::TelosMainnet => 0x06,
        }
    }

    pub fn from_byte(byte: &Byte) -> Result<Self> {
        match byte {
            0u8 => Ok(BlockchainId::EthereumMainnet),
            1u8 => Ok(BlockchainId::EthereumRinkeby),
            2u8 => Ok(BlockchainId::EthereumRopsten),
            3u8 => Ok(BlockchainId::BitcoinMainnet),
            4u8 => Ok(BlockchainId::BitcoinTestnet),
            5u8 => Ok(BlockchainId::EosMainnet),
            6u8 => Ok(BlockchainId::TelosMainnet),
            _ => Err(format!("✘ Unrecognized version byte for `BlockchainId`: {:?}", byte).into()),
        }
    }
}

impl fmt::Display for BlockchainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockchainId::EosMainnet => write!(f, "Eos Mainnet"),
            BlockchainId::TelosMainnet => write!(f, "Telos Mainnet"),
            BlockchainId::BitcoinMainnet => write!(f, "Bitcoin Mainnet"),
            BlockchainId::BitcoinTestnet => write!(f, "Bitcoin Testnet"),
            BlockchainId::EthereumMainnet => write!(f, "Ethereum Mainnet"),
            BlockchainId::EthereumRinkeby => write!(f, "Ethereum Rinkeby Testnet"),
            BlockchainId::EthereumRopsten => write!(f, "Ethereum Ropsten Testnet"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_all_blockchain_ids() -> Vec<BlockchainId> {
        vec![
            BlockchainId::EthereumMainnet,
            BlockchainId::EthereumRopsten,
            BlockchainId::EthereumRinkeby,
            BlockchainId::BitcoinMainnet,
            BlockchainId::BitcoinTestnet,
            BlockchainId::EosMainnet,
            BlockchainId::TelosMainnet,
        ]
    }

    #[test]
    fn should_perform_blockchain_ids_bytes_round_trip() {
        let blockchain_ids = get_all_blockchain_ids();
        blockchain_ids.iter().for_each(|id| {
            let byte = id.as_byte();
            let result = BlockchainId::from_byte(&byte).unwrap();
            assert_eq!(&result, id);
        });
    }
}
