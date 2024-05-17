use std::sync::Arc;

use common_eth::EthSubmissionMaterial;
use derive_more::{Constructor, Deref, DerefMut};
use eth_trie::{EthTrie, MemoryDB, Trie};

use super::MerkleError;

#[derive(Debug, Constructor, Deref, DerefMut)]
pub struct MerkleTree(EthTrie<MemoryDB>);

impl TryFrom<&EthSubmissionMaterial> for MerkleTree {
    type Error = MerkleError;

    fn try_from(sub_mat: &EthSubmissionMaterial) -> Result<Self, Self::Error> {
        // NOTE: https://github.com/carver/eth-trie.rs/blob/94ad815505c4a1dce97d6f30a052446ce3b2abfb/src/db.rs#L52
        let db = Arc::new(MemoryDB::new(true));
        let mut trie = EthTrie::new(db);

        for receipt in sub_mat.receipts.iter() {
            let (k, v) = receipt.get_rlp_encoded_index_and_rlp_encoded_receipt_tuple()?;
            trie.insert(&k, &v)?;
        }

        Ok(Self::new(trie))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_sub_mat_n;

    #[test]
    fn should_calculate_merkle_root() {
        let sub_mat = get_sample_sub_mat_n(1);
        let mut merkle_tree = MerkleTree::try_from(&sub_mat).unwrap();
        let root_hash = merkle_tree.root_hash().unwrap();
        let expected_root_hash = sub_mat.receipts_root.unwrap();
        assert_eq!(root_hash.as_bytes(), expected_root_hash.as_bytes());
    }
}
