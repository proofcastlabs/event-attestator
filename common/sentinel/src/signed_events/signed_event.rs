use common::{sha256_hash_bytes, types::Bytes, utils::left_pad_bytes_with_zeroes};
use common_chain_ids::EthChainId;
use common_eth::{EthLog, EthPrivateKey, EthSigningCapabilities};
use common_metadata::MetadataChainId;
use common_network_ids::ProtocolId;
use derive_getters::Getters;
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use super::{EventIdError, SignedEventError, SignedEventVersion};
use crate::MerkleProof;

#[derive(Debug, Clone, Eq, PartialEq, Default, Serialize, Deserialize, Getters)]
pub struct SignedEvent {
    version: SignedEventVersion,
    protocol: ProtocolId,
    origin: EthChainId,
    log: EthLog,
    tx_id_hash: EthHash,
    block_id_hash: EthHash,
    // NOTE: String in case format changes, plus can't auto derive ser/de on [u8; 65]
    // It's an option so we can create the struct with no signature then add it later.
    event_payload: Option<String>,
    event_id: Option<String>,
    signature: Option<String>,
    public_key: String,
}

const CHAIN_ID_PADDING: usize = 32;
const EVENT_ADDRESS_PADDING: usize = 32;

impl SignedEvent {
    pub(super) fn new(
        metadata_chain_id: MetadataChainId,
        log: EthLog,
        tx_id_hash: EthHash,
        block_id_hash: EthHash,
        pk: &EthPrivateKey,
        // FIXME reintroduce
        _merkle_proof: MerkleProof,
    ) -> Result<Self, SignedEventError> {
        let public_key = pk.to_public_key().public_key.to_string();
        let mut signed_event = Self {
            version: SignedEventVersion::current(),
            protocol: metadata_chain_id.to_protocol_id().into(),
            origin: metadata_chain_id.to_eth_chain_id()?,
            log,
            tx_id_hash,
            block_id_hash,
            event_payload: None,
            event_id: None,
            signature: None,
            public_key,
        };
        let event_payload = signed_event.get_event_payload()?;
        signed_event.event_payload = Some(hex::encode(event_payload));
        let event_id_preimage = signed_event.get_event_id_preimage()?;
        let event_id = EventId(sha256_hash_bytes(&event_id_preimage));
        signed_event.event_id = Some(event_id.to_string());
        let sig = pk.sha256_hash_and_sign_msg_with_normalized_parity(&event_id_preimage)?;
        signed_event.signature = Some(sig.to_string());
        Ok(signed_event)
    }

    fn get_event_payload(&self) -> Result<Bytes, SignedEventError> {
        let address = left_pad_bytes_with_zeroes(self.log.address.as_bytes(), EVENT_ADDRESS_PADDING);
        let mut topics = self
            .log
            .topics
            .iter()
            .map(|t| t.as_bytes().to_vec())
            .collect::<Vec<_>>();
        while topics.len() < 4 {
            topics.push(EthHash::zero().as_bytes().to_vec());
        }

        Ok([address, topics.concat(), self.log.data.to_vec()].concat())
    }

    fn get_event_id_preimage(&self) -> Result<Bytes, EventIdError> {
        let event_payload = self.event_payload.as_ref().ok_or(EventIdError::EncodedEventIsNone)?;
        let pre_image = [
            self.version.as_bytes(),
            &[self.protocol.into()],
            &left_pad_bytes_with_zeroes(&self.origin.to_bytes()?, CHAIN_ID_PADDING),
            self.block_id_hash.as_bytes(),
            self.tx_id_hash.as_bytes(),
            &hex::decode(event_payload)?,
        ]
        .concat();

        Ok(pre_image)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventId(pub Bytes);

impl std::fmt::Display for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ethereum_types::Address as EthAddress;

    use super::*;

    #[test]
    fn should_create_a_signed_event_correctly() {
        let metadata_chain_id = MetadataChainId::EthereumMainnet;
        let address = EthAddress::from_str("0x87415715056da7a5eb1a30e53c4f4d20b44db71d").unwrap();
        let topics = vec![
            EthHash::from_str("0x9b706941b48091a1c675b439064f40b9d43c577d9c7134cce93179b9b0bf2a52").unwrap(),
            EthHash::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
        ];
        let data = hex::decode("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000ea000000000000000000000000000000000000000000000000000000000000000000000000000000000000000051a240271ab8ab9f9a21c82d9a85396b704e164d0000000000000000000000000000000000000000000000000000000000007a6a00000000000000000000000000000000000000000000000000000000000026fc0000000000000000000000002b5ad5c4795c026514f8317c7a215e218dccd6cf000000000000000000000000000000000000000000000000000000000000002a30783638313345623933363233373245454636323030663362316462433366383139363731634241363900000000000000000000000000000000000000000000").unwrap();
        let log = EthLog::new(address, topics, data);
        let block_id_hash =
            EthHash::from_str("0x658d5ae6a577714c7507e7b5911d26429280d6a0922a2be3f4502d577985527a").unwrap();
        let tx_id_hash =
            EthHash::from_str("0x9b3b567ec90fc3a263f1784f57f942ac52ab4e609c23ba794de944fc1b512d34").unwrap();
        let pk = EthPrivateKey::from_str("dfcc79a57e91c42d7eea05f82a08bd1b7e77f30236bb7c56fe98d3366a1929c4").unwrap();
        let merkle_proof = MerkleProof::new(vec![vec![]]);
        let result = SignedEvent::new(metadata_chain_id, log, tx_id_hash, block_id_hash, &pk, merkle_proof).unwrap();

        let expected_signature = "5b838b1283851a1fa35ba79ea39bb74b0bf7ec7d3c0bcb96d3879e28d291c8e348a74ff321b0e02fa3960fc1fec2ddc2e49738a77d0f9f1a596312b6bb03b8f01c".to_string();

        assert_eq!(result.signature.unwrap(), expected_signature);
    }
}
