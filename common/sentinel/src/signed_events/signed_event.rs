use common::{crypto_utils::sha256_hash_bytes, types::Bytes, utils::left_pad_bytes_with_zeroes};
use common_chain_ids::EthChainId;
use common_eth::{EthLog, EthPrivateKey, EthSigningCapabilities};
use common_metadata::MetadataChainId;
use common_network_ids::ProtocolId;
use derive_getters::Getters;
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
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
        let event_id = signed_event.calculate_event_id()?;
        signed_event.event_id = Some(event_id.to_string());
        let sig = pk.sha256_hash_and_sign_msg_with_normalized_parity(&event_id.0)?;
        signed_event.signature = Some(sig.to_string());
        Ok(signed_event)
    }

    fn get_event_payload(&self) -> Result<Bytes, SignedEventError> {
        let address = left_pad_bytes_with_zeroes(self.log.address.as_bytes(), EVENT_ADDRESS_PADDING);
        let topics = self
            .log
            .topics
            .iter()
            .map(|t| t.as_bytes().to_vec())
            .collect::<Vec<_>>()
            .concat();
        let event_bytes = eth_abi_decode(&[EthAbiParamType::Tuple(vec![EthAbiParamType::Bytes])], &self.log.data)?;
        let event_bytes = match &event_bytes[0] {
            EthAbiToken::Tuple(value) => match &value[0] {
                EthAbiToken::Bytes(bytes) => Ok(bytes.to_vec()),
                _ => Err(SignedEventError::LogDataError("bad log data format".to_string())),
            },
            _ => Err(SignedEventError::LogDataError("bad log data format".to_string())),
        }?;

        Ok([address, sha256_hash_bytes(&topics), event_bytes].concat())
    }

    fn calculate_event_id(&self) -> Result<EventId, EventIdError> {
        let event_payload = self.event_payload.as_ref().ok_or(EventIdError::EncodedEventIsNone)?;
        let event_id = [
            self.version.as_bytes(),
            &[self.protocol.into()],
            &left_pad_bytes_with_zeroes(&self.origin.to_bytes()?, CHAIN_ID_PADDING),
            self.block_id_hash.as_bytes(),
            self.tx_id_hash.as_bytes(),
            &hex::decode(event_payload)?,
        ]
        .concat();

        Ok(EventId(event_id))
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
        let address = EthAddress::from_str("0x2946259E0334f33A064106302415aD3391BeD384").unwrap();
        let topics = vec![
            EthHash::from_str("0x289ca1b08b8acb2ac02d0c5e8610fd8c0222f15b3089f0b7e7f7f284a23325aa").unwrap(),
            EthHash::from_str("0x0000000000000000000000000000000000000000000000000000000000000000").unwrap(),
        ];
        let event_bytes = hex::decode("0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000051a240271ab8ab9f9a21c82d9a85396b704e164d0000000000000000000000000000000000000000000000000000000000007a6a00000000000000000000000000000000000000000000000000000000000026fc2b5ad5c4795c026514f8317c7a215e218dccd6cf00000000000000000000000000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000002a307836383133456239333632333732454546363230306633623164624333663831393637316342413639000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap();
        let log = EthLog::new(address, topics, event_bytes);
        let block_id_hash =
            EthHash::from_str("0xa880cb2ab67ec9140db0f6de238b34d4108f6fab99315772ee987ef9002e0e63").unwrap();
        let tx_id_hash =
            EthHash::from_str("0x11365bbee18058f12c27236e891a66999c4325879865303f785854e9169c257a").unwrap();
        let pk = EthPrivateKey::from_str("dfcc79a57e91c42d7eea05f82a08bd1b7e77f30236bb7c56fe98d3366a1929c4").unwrap();
        let merkle_proof = MerkleProof::new(vec![vec![]]);
        let result = SignedEvent::new(metadata_chain_id, log, tx_id_hash, block_id_hash, &pk, merkle_proof).unwrap();

        let expected_signature = "aeaf013bed3e5faa833f035a34f34f06b17fc853c8becdfc67031fffbaf6a42d7bbbde18659cf028738d3155635d51fe84b2bf4cab5dfb37c937e37d4e9fcaee1c".to_string();

        assert_eq!(result.signature.unwrap(), expected_signature);
    }
}
