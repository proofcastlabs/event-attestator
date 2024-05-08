use common_eth::{EthPrivateKey, EthReceipts};
use common_metadata::MetadataChainId;
use derive_more::{Constructor, Deref, DerefMut};
use ethereum_types::H256 as EthHash;
use serde::{Deserialize, Serialize};

use super::{SignedEvent, SignedEventVersion};
use crate::{MerkleProof, NetworkConfig, SentinelError};

#[derive(Debug, Clone, Default, Serialize, Deserialize, Constructor, Deref, DerefMut)]
pub struct SignedEvents(Vec<SignedEvent>);

impl TryFrom<(&MetadataChainId, &EthPrivateKey, &EthReceipts, &NetworkConfig)> for SignedEvents {
    type Error = SentinelError;

    fn try_from(
        (metadata_chain_id, private_key, receipts, network_config): (
            &MetadataChainId,
            &EthPrivateKey,
            &EthReceipts,
            &NetworkConfig,
        ),
    ) -> Result<Self, Self::Error> {
        // NOTE: The tuple is everything we need to parse signed events from a block

        let version = SignedEventVersion::current();
        let target_tx_hashes = network_config
            .events()
            .iter()
            .map(|_event| {
                todo!("use the `event.address()` and `event.topic()` params to parse relevant txs from receipts")
            })
            .collect::<Result<Vec<EthHash>, Self::Error>>()?;

        let merkle_proofs = target_tx_hashes
            .iter()
            .map(|tx_hash| MerkleProof::try_from((receipts, tx_hash)))
            .collect::<Result<Vec<MerkleProof>, Self::Error>>()?;

        todo!("continue parsing the various bits to create the vec of `SignedEvent` below");
        let events = vec![];

        Ok(Self::new(events))
    }
}
