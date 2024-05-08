use derive_more::Constructor;
use serde::{Serialize, Deserialize};
use derive_getters::Getters;
use common_eth::EthLog;
use common_network_ids::NetworkId;

#[derive(Debug, Clone, Serialize, Deserialize, Getters, Constructor)]
pub struct SignedEvent {
    log: EthLog,
    // NOTE: String in case format changes, plus can't auto derive ser/de on [u8; 65]
    signature: String,
    network_id: NetworkId,
}
