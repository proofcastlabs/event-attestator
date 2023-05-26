use common::types::{Bytes, Result};
use common_eth::MAX_BYTES_FOR_ETH_USER_DATA;
use common_metadata::{Metadata, MetadataAddress, MetadataProtocolId};

use crate::algo::int_tx_info::IntOnAlgoIntTxInfo;

impl IntOnAlgoIntTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        let user_data = if self.user_data.len() > MAX_BYTES_FOR_ETH_USER_DATA {
            info!(
                "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes",
                MAX_BYTES_FOR_ETH_USER_DATA
            );
            vec![]
        } else {
            info!(
                "✔ User data to be wrapped in metadata: 0x{}",
                hex::encode(&self.user_data)
            );
            self.user_data.clone()
        };
        let metadata = Metadata::new_v3(
            &user_data,
            &MetadataAddress::new(&self.token_sender.to_string(), &self.origin_chain_id)?,
            &MetadataAddress::new(&self.destination_address, &self.destination_chain_id)?,
            None,
            None,
        );
        Ok(metadata)
    }

    pub fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}
