use crate::SentinelError;
use ethereum_types::H256 as EthHash;
use common_eth::EthReceipts;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MerkleProof(Vec<EthHash>);

impl TryFrom<(EthHash, EthReceipts, EthHash)> for MerkleProof {
    type Error = SentinelError;

    fn try_from((_target_root_hash, _receipts, _target_tx_hash): (EthHash, EthReceipts, EthHash)) -> Result<Self, Self::Error> {
        // NOTE: Given a set of tx receipts, a target root hash and a target tx hash, a merkle
        // proof can be built. Note that the block may already have been validated elsewhere
        // meaning validating this set against the root hash may be redundant work and a place for
        // optimisation. Here's a link to the eos action proof makers merkle proof creation fxn for
        // reference: https://github.com/pnetwork-association/eos-action-proof-maker/blob/95ae7167ae22da0390b8fafc5a178c41f77e0dad/src/eos_merkle_utils.rs#L86
        // There are likely other crates that can make this easy to do too, eg reth
        // Pass in args probably ought to be a struct too for more clarity rather than this tuple.
        todo!("make merkle proof")
    }
}
