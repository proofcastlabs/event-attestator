use crate::{
    btc_on_int::btc::int_tx_info::BtcOnIntIntTxInfo,
    chains::eth::eth_constants::MAX_BYTES_FOR_ETH_USER_DATA,
    metadata::{
        metadata_address::MetadataAddress,
        metadata_chain_id::MetadataChainId,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::ToMetadata,
        Metadata,
    },
    types::{Bytes, Result},
};

impl ToMetadata for BtcOnIntIntTxInfo {
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
        let origin_chain_id = MetadataChainId::from_bytes(&self.origin_chain_id)?;
        let destination_chain_id = MetadataChainId::from_bytes(&self.destination_chain_id)?;
        let metadata = Metadata::new_v3(
            &user_data,
            &MetadataAddress::new(self.originating_tx_address.clone(), origin_chain_id),
            &MetadataAddress::new(self.destination_address.clone(), destination_chain_id),
            None,
            None,
        );
        Ok(metadata)
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}
