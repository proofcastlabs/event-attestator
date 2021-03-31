use std::fmt;

use crate::types::{Byte, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetadataProtocolId {
    Bitcoin,
    Ethereum,
    Eos,
}

impl MetadataProtocolId {
    pub fn to_byte(&self) -> Byte {
        match self {
            MetadataProtocolId::Ethereum => 0x00,
            MetadataProtocolId::Bitcoin => 0x01,
            MetadataProtocolId::Eos => 0x02,
        }
    }

    pub fn from_byte(byte: &Byte) -> Result<Self> {
        match byte {
            0u8 => Ok(MetadataProtocolId::Ethereum),
            1u8 => Ok(MetadataProtocolId::Bitcoin),
            2u8 => Ok(MetadataProtocolId::Eos),
            _ => Err(format!("✘ Unrecognized version byte for `MetadataProtocolId`: {:?}", byte).into()),
        }
    }

    pub fn to_symbol(&self) -> String {
        let s = match self {
            MetadataProtocolId::Ethereum => "ETH",
            MetadataProtocolId::Bitcoin => "BTC",
            MetadataProtocolId::Eos => "EOS",
        };
        s.to_string()
    }

    #[cfg(test)]
    fn get_all() -> Vec<Self> {
        // FIXME / TODO How to ensure this contains all variants?
        vec![
            MetadataProtocolId::Bitcoin,
            MetadataProtocolId::Eos,
            MetadataProtocolId::Ethereum,
        ]
    }
}

impl fmt::Display for MetadataProtocolId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MetadataProtocolId::Ethereum => write!(f, "Ethereum"),
            MetadataProtocolId::Bitcoin => write!(f, "Bitcoin"),
            MetadataProtocolId::Eos => write!(f, "Eos"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_perform_metadata_protocol_ids_bytes_round_trip() {
        MetadataProtocolId::get_all().iter().for_each(|id| {
            let byte = id.to_byte();
            let result = MetadataProtocolId::from_byte(&byte).unwrap();
            assert_eq!(&result, id);
        });
    }
}
