use ethereum_types::H256;

use crate::types::Result;

pub trait ChainId {
    fn keccak_hash(&self) -> Result<H256>;
}
