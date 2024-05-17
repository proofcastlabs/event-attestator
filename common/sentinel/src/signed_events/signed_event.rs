use common::types::Bytes;
use common_chain_ids::EthChainId;
use common_eth::{EthLog, EthPrivateKey, EthSigningCapabilities};
use common_metadata::{MetadataChainId, MetadataProtocolId};
use derive_getters::Getters;
use derive_more::Deref;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use super::{SignedEventError, SignedEventVersion};
use crate::MerkleProof;

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Getters)]
pub struct SignedEvent {
    log: EthLog,
    block_hash: EthHash,
    // NOTE: String in case format changes, plus can't auto derive ser/de on [u8; 65]
    // It's an option so we can create the struct with no signature then add it later.
    encoded_event: Option<String>,
    signature: Option<String>,
    event_id: String,
    public_key: String,
    version: SignedEventVersion,
}

fn calculate_event_id() -> EventId {
    // FIXME sha256(protocol, protocol_chain_id, blockhash, unique_event_identifier_such_as_merklepathonevm)
    // Currently, arbitrary bytes
    EventId([0xab, 0xcd, 0xee, 0xff])
}

impl SignedEvent {
    pub(super) fn new(
        log: EthLog,
        block_hash: EthHash,
        // FIXME reintroduce
        _merkle_proof: MerkleProof,
        _metadata_chain_id: MetadataChainId,
        pk: &EthPrivateKey,
    ) -> Result<Self, SignedEventError> {
        let event_id = calculate_event_id().to_string();
        let public_key = pk.to_public_key().public_key.to_string();
        let mut signed_event = Self {
            log,
            block_hash,
            encoded_event: None,
            signature: None,
            event_id,
            public_key,
            version: SignedEventVersion::current(),
        };
        let encoded_event = signed_event.encode();
        signed_event.encoded_event = Some(encoded_event.to_string());
        let sig = pk.sha256_hash_and_sign_msg(encoded_event.as_slice())?;
        signed_event.signature = Some(sig.to_string());
        Ok(signed_event)
    }

    fn encode(&self) -> EncodedEvent {
        EncodedEvent(
            [
                self.version.as_bytes(),
                &[MetadataProtocolId::Ethereum.to_byte()],
                EthChainId::Mainnet.to_bytes().as_ref().unwrap(),
                &self.event_id.as_bytes(),
                &self.log.data.len().to_le_bytes(),
                self.log.data.as_ref(),
            ]
            .concat(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventId(pub [u8; 4]);

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deref)]
pub struct EncodedEvent(pub Bytes);

impl std::fmt::Display for EncodedEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}
