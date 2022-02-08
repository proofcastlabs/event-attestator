use ethereum_types::H256 as KeccakHash;
use rust_algorand::AlgorandGenesisId;

use crate::{
    crypto_utils::keccak_hash_bytes,
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

    fn from_genesis_id(gensis_id: &AlgorandGenesisId) -> Result<Self> {
        // FIXME. Can also use the AlgorandGenesisId::from_str method to re-implement that here if
        // we want?
        unimplemented!()
    }
}
