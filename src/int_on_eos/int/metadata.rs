use crate::{
    int_on_eos::int::eos_tx_info::IntOnEosEosTxInfo,
    metadata::{
        metadata_address::MetadataAddress,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::ToMetadata,
        Metadata,
    },
    types::{Bytes, Result},
};

impl ToMetadata for IntOnEosEosTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        Ok(Metadata::new(
            // FIXME Do we need v3??
            &self.user_data,
            &MetadataAddress::new_from_eth_address(&self.token_sender, &self.origin_chain_id)?,
        ))
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Eos)
    }
}
