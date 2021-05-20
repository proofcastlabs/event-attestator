use std::fmt;

use ethereum_types::H256 as KeccakHash;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use crate::{
    crypto_utils::keccak_hash_bytes,
    metadata::{metadata_chain_id::MetadataChainId, metadata_traits::ToMetadataChainId},
    traits::ChainId,
    types::{Byte, Bytes, Result},
    utils::decode_hex_with_err_msg,
};

#[derive(Clone, Debug, PartialEq, Eq, EnumIter)]
pub enum EosChainId {
    EosMainnet,
    TelosMainnet,
    EosJungleTestnet,
    UltraMainnet,
    FioMainnet,
}

impl ChainId for EosChainId {
    fn keccak_hash(&self) -> Result<KeccakHash> {
        Ok(keccak_hash_bytes(&self.to_bytes()))
    }
}

impl ToMetadataChainId for EosChainId {
    fn to_metadata_chain_id(&self) -> MetadataChainId {
        match self {
            Self::EosMainnet => MetadataChainId::EosMainnet,
            Self::FioMainnet => MetadataChainId::FioMainnet,
            Self::TelosMainnet => MetadataChainId::TelosMainnet,
            Self::UltraMainnet => MetadataChainId::UltraMainnet,
            Self::EosJungleTestnet => MetadataChainId::EosJungleTestnet,
        }
    }
}

lazy_static! {
    pub static ref EOS_MAINNET_BYTES: Bytes =
        hex::decode("aca376f206b8fc25a6ed44dbdc66547c36c6c33e3a119ffbeaef943642f0e906")
            .expect("✘ Invalid hex in `EOS_MAINNET_BYTES`");
    pub static ref TELOS_MAINNET_BYTES: Bytes =
        hex::decode("4667b205c6838ef70ff7988f6e8257e8be0e1284a2f59699054a018f743b1d11")
            .expect("✘ Invalid hex in `TELOS_MAINNET_BYTES`");
    pub static ref EOS_JUNGLE_TESTNET_BYTES: Bytes =
        hex::decode("e70aaab8997e1dfce58fbfac80cbbb8fecec7b99cf982a9444273cbc64c41473")
            .expect("✘ Invalid hex in `EOS_JUNGLE_TESTNET_BYTES`");
    pub static ref ULTRA_MAINNET_BYTES: Bytes =
        hex::decode("9d4ce4f29989020912def3bd130481ad4d34ab7a6b2cae969a62b11b86f32d7f")
            .expect("✘ Invalid hex in `ULTRA_MAINNET_BYTES`");
    pub static ref FIO_MAINNET_BYTES: Bytes =
        hex::decode("21dcae42c0182200e93f954a074011f9048a7624c6fe81d3c9541a614a88bd1c")
            .expect("✘ Invalid hex in `FIO_MAINNET_BYTES`");
}

impl EosChainId {
    pub fn from_str(s: &str) -> Result<Self> {
        decode_hex_with_err_msg(s, &format!("`EosChainId` error! Invalid hex: 0x{}", s))
            .and_then(|ref bytes| Self::from_bytes(bytes))
    }

    pub fn to_hex(&self) -> String {
        match self {
            Self::EosMainnet => hex::encode(&*EOS_MAINNET_BYTES),
            Self::TelosMainnet => hex::encode(&*TELOS_MAINNET_BYTES),
            Self::EosJungleTestnet => hex::encode(&*EOS_JUNGLE_TESTNET_BYTES),
            Self::UltraMainnet => hex::encode(&*ULTRA_MAINNET_BYTES),
            Self::FioMainnet => hex::encode(&*FIO_MAINNET_BYTES),
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

    pub fn to_bytes(&self) -> Bytes {
        match self {
            Self::EosMainnet => EOS_MAINNET_BYTES.to_vec(),
            Self::TelosMainnet => TELOS_MAINNET_BYTES.to_vec(),
            Self::EosJungleTestnet => EOS_JUNGLE_TESTNET_BYTES.to_vec(),
            Self::UltraMainnet => ULTRA_MAINNET_BYTES.to_vec(),
            Self::FioMainnet => FIO_MAINNET_BYTES.to_vec(),
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
            Self::EosJungleTestnet => write!(f, "EOS Jungle Testnet: 0x{}", self.to_hex()),
            Self::UltraMainnet => write!(f, "Ultra Mainnet: 0x{}", self.to_hex()),
            Self::FioMainnet => write!(f, "FIO Mainnet: 0x{}", self.to_hex()),
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
