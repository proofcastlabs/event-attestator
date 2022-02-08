use ethereum_types::H256 as KeccakHash;
use rust_algorand::AlgorandGenesisId;

use crate::{
    crypto_utils::keccak_hash_bytes,
    metadata::metadata_chain_id::MetadataChainId,
    traits::ChainId,
    types::{Bytes, Result},
};

impl ChainId for AlgoChainId {
    fn keccak_hash(&self) -> Result<KeccakHash> {
        Ok(keccak_hash_bytes(&self.to_bytes()?))
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum AlgoChainId {
    Mainnet,
}

impl Default for AlgoChainId {
    fn default() -> Self {
        Self::Mainnet
    }
}

impl AlgoChainId {
    fn to_bytes(&self) -> Result<Bytes> {
        match self {
            Self::Mainnet => Ok(AlgorandGenesisId::Mainnet.hash()?.to_bytes()),
        }
    }

    pub fn from_genesis_id(gensis_id: &AlgorandGenesisId) -> Result<Self> {
        match gensis_id {
            AlgorandGenesisId::Mainnet => Ok(Self::Mainnet),
            _ => Err(format!("Unsupported `AlgorandGenesisId` {}!", gensis_id).into()),
        }
    }

    pub fn to_metadata_chain_id(&self) -> MetadataChainId {
        match self {
            Self::Mainnet => MetadataChainId::AlgorandMainnet,
        }
    }
}
