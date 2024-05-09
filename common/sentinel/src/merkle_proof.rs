use std::sync::Arc;

use common_eth::EthReceipts;
use derive_more::Constructor;
use eth_trie::{EthTrie, MemoryDB, Trie};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MerkleProofError {
    #[error("cannot make proof, target key is not in trie")]
    NoKeyToProve,

    #[error("trie error: {0}")]
    Trie(#[from] eth_trie::TrieError),

    #[error("common error: {0}")]
    Common(#[from] common::CommonError),
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, Constructor)]
pub struct MerkleProof(Vec<Vec<u8>>);

impl TryFrom<(&EthReceipts, &EthHash)> for MerkleProof {
    type Error = MerkleProofError;

    fn try_from((receipts, target_tx_hash): (&EthReceipts, &EthHash)) -> Result<Self, Self::Error> {
        // NOTE: the root will be calculated fomr the set of receipts. You may want to add in a
        // target root in order to validate this is the correct root, though that should have been
        // validated elsewhere and may be redundant. However there's no guarantee of this. At least
        // if we don't add a target root, the proof created will either be correct w/r/t the
        // desired block or not and thus will fail, which is likely the most robust behaviour we'd
        // want.

        let mut maybe_key_to_prove: Option<Vec<u8>> = None;
        // NOTE: https://github.com/carver/eth-trie.rs/blob/94ad815505c4a1dce97d6f30a052446ce3b2abfb/src/db.rs#L52
        let db = Arc::new(MemoryDB::new(true));
        let mut trie = EthTrie::new(db);

        for (i, receipt) in receipts.iter().enumerate() {
            let (k, v) = receipt.get_rlp_encoded_index_and_rlp_encoded_receipt_tuple()?;
            if &receipt.transaction_hash == target_tx_hash {
                maybe_key_to_prove = Some(k.clone());
            };
            trie.insert(&k, &v)?;
        }

        let key_to_prove = if maybe_key_to_prove.is_none() {
            return Err(MerkleProofError::NoKeyToProve);
        } else {
            maybe_key_to_prove.expect("this never to fail due to above")
        };

        // NOTE: Proof format contains all encoded nodes on the path to the value at key. The
        // value itself is also included in the last node.  We don't have to care about the
        // case where there's no value for the key since we've handled it above.
        // Docs here: https://github.com/carver/eth-trie.rs/blob/94ad815505c4a1dce97d6f30a052446ce3b2abfb/src/trie.rs#L34
        Ok(Self::new(trie.get_proof(&key_to_prove)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_get_merkle_proof() {
        todo!("write this test");
    }
}
