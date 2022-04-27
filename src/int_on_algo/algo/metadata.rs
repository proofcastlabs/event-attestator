use crate::{
    chains::eth::eth_constants::MAX_BYTES_FOR_ETH_USER_DATA,
    int_on_algo::algo::int_tx_info::IntOnAlgoIntTxInfo,
    metadata::{
        metadata_address::MetadataAddress,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::ToMetadata,
        Metadata,
    },
    types::{Bytes, Result},
};

impl ToMetadata for IntOnAlgoIntTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        let user_data = if self.user_data.len() > MAX_BYTES_FOR_ETH_USER_DATA {
            info!(
                "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes",
                MAX_BYTES_FOR_ETH_USER_DATA
            );
            vec![]
        } else {
            self.user_data.clone()
        };
        let metadata = Metadata::new_v3(
            &user_data,
            &MetadataAddress::new(self.token_sender.to_string(), self.origin_chain_id)?,
            &MetadataAddress::new(self.destination_address.clone(), self.destination_chain_id)?,
            None,
            None,
        );
        Ok(metadata)
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}
