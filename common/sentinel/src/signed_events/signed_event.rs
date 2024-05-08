use derive_more::Constructor;
use serde::{Serialize, Deserialize};
use derive_getters::Getters;
use common_eth::EthLog;
use ethereum_types::H256 as EthHash;
use common_metadata::MetadataChainId;
use super::SignedEventVersion;
use crate::MerkleProof;

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Getters, Constructor)]
pub struct SignedEvent {
    log: EthLog,
    // NOTE: String in case format changes, plus can't auto derive ser/de on [u8; 65]
    signature: String,
    block_hash: EthHash,
    merkle_proof: MerkleProof,
    version: SignedEventVersion,
    // NOTE: gitmp needs this instead of the v3 NetworkId
    metadata_chain_id: MetadataChainId,
}
