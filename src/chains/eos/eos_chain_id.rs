use std::fmt;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    metadata::blockchain_chain_id::BlockchainChainId,
    types::{Byte, Bytes, Result},
};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum EosChainId {
    EosMainnet,
    TelosMainnet,
}

lazy_static! {
    pub static ref EOS_MAINNET_BYTES: Bytes =
        hex::decode("aca376f206b8fc25a6ed44dbdc66547c36c6c33e3a119ffbeaef943642f0e906")
            .expect("✘ Invalid hex in `EOS_MAINNET_BYTES`");
    pub static ref TELOS_MAINNET_BYTES: Bytes =
        hex::decode("4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11")
            .expect("✘ Invalid hex in `TELOS_MAINNET_BYTES`");
}

impl EosChainId {
    fn to_blockchain_chain_id(&self) -> Result<BlockchainChainId> {
        match self {
            Self::EosMainnet => Ok(BlockchainChainId::EosMainnet),
            Self::TelosMainnet => Ok(BlockchainChainId::TelosMainnet),
        }
    }

    fn to_hex(&self) -> String {
        match self {
            Self::EosMainnet => hex::encode(&*EOS_MAINNET_BYTES),
            Self::TelosMainnet => hex::encode(&*TELOS_MAINNET_BYTES),
        }
    }

    fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        info!("✔ Getting `EosChainId` from bytes: 0x{}", hex::encode(bytes));
        let maybe_self = Self::get_all()
            .iter()
            .map(|eos_chain_id| {
                let eos_chain_id_bytes = eos_chain_id.to_bytes();
                if eos_chain_id_bytes == bytes {
                    Some(eos_chain_id.clone())
                } else {
                    None
                }
            })
            .filter(Option::is_some)
            .collect::<Vec<Option<Self>>>();
        match maybe_self.len() {
            1 => maybe_self[0]
                .clone()
                .ok_or_else(|| "Failed to unwrap `maybe_self` from option!".into()),
            _ => Err(format!("Unrecognized bytes for `EosChainId`: 0x{}", hex::encode(bytes)).into()),
        }
    }

    fn to_bytes(&self) -> Bytes {
        match self {
            Self::EosMainnet => EOS_MAINNET_BYTES.to_vec(),
            Self::TelosMainnet => TELOS_MAINNET_BYTES.to_vec(),
        }
    }

    fn get_all() -> Vec<Self> {
        Self::iter().collect()
    }
}

impl fmt::Display for EosChainId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::EosMainnet => write!(f, "EOS Mainnet: 0x{}", self.to_hex()),
            Self::TelosMainnet => write!(f, "Telos Mainnet: 0x{}", self.to_hex()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_make_bytes_roundtrip_for_all_eos_chain_ids() {
        let ids = EosChainId::get_all();
        let vec_of_bytes = ids.iter().map(|id| id.to_bytes()).collect::<Vec<Bytes>>();
        let result = vec_of_bytes
            .iter()
            .map(|ref bytes| EosChainId::from_bytes(bytes))
            .collect::<Result<Vec<EosChainId>>>()
            .unwrap();
        assert_eq!(result, ids);
    }
}
