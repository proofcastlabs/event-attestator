use common_eth::EthReceipts;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use crate::SentinelError;

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct MerkleProof(Vec<EthHash>);

impl TryFrom<(&EthReceipts, &EthHash)> for MerkleProof {
    type Error = SentinelError;

    fn try_from((_receipts, _target_tx_hash): (&EthReceipts, &EthHash)) -> Result<Self, Self::Error> {
        // NOTE: the root will be calculated fomr the set of receipts. You may want to add in a
        // target root in order to validate this is the correct root, though that should have been
        // validated elsewhere and may be redundant. However there's no guarantee of this. At least
        // if we don't add a target root, the proof created will either be correct w/r/t the
        // desired block or not and thus will fail, which is likely the most robust behaviour we'd
        // want.

        // NOTE: Here's a link to the eos action proof makers merkle proof creation fxn for
        // reference: https://github.com/pnetwork-association/eos-action-proof-maker/blob/95ae7167ae22da0390b8fafc5a178c41f77e0dad/src/eos_merkle_utils.rs#L86
        // There are likely other crates that can make this easy to do too, eg reth
        // Pass in args probably ought to be a struct too for more clarity rather than this tuple.
        todo!("make merkle proof")
    }
}
