use std::fmt;

use crate::types::{Byte, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockchainProtocolId {
    Bitcoin,
    Ethereum,
    Eos,
}

impl BlockchainProtocolId {
    pub fn as_byte(&self) -> Byte {
        match self {
            BlockchainProtocolId::Ethereum => 0x00,
            BlockchainProtocolId::Bitcoin => 0x01,
            BlockchainProtocolId::Eos => 0x02,
        }
    }

    pub fn from_byte(byte: &Byte) -> Result<Self> {
        match byte {
            0u8 => Ok(BlockchainProtocolId::Ethereum),
            1u8 => Ok(BlockchainProtocolId::Bitcoin),
            2u8 => Ok(BlockchainProtocolId::Eos),
            _ => Err(format!("✘ Unrecognized version byte for `BlockchainProtocolId`: {:?}", byte).into()),
        }
    }

    #[cfg(test)]
    fn get_all() -> Vec<Self> {
        // TODO How to ensure this contains all variants?
        vec![
            BlockchainProtocolId::Bitcoin,
            BlockchainProtocolId::Eos,
            BlockchainProtocolId::Ethereum,
        ]
    }
}

impl fmt::Display for BlockchainProtocolId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockchainProtocolId::Ethereum => write!(f, "Ethereum"),
            BlockchainProtocolId::Bitcoin => write!(f, "Bitcoin"),
            BlockchainProtocolId::Eos => write!(f, "Eos"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_perform_blockchain_protocol_ids_bytes_round_trip() {
        BlockchainProtocolId::get_all().iter().for_each(|id| {
            let byte = id.as_byte();
            let result = BlockchainProtocolId::from_byte(&byte).unwrap();
            assert_eq!(&result, id);
        });
    }
}
