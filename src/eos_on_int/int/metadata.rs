use crate::{
    eos_on_int::int::eos_tx_info::EosOnIntEosTxInfo,
    metadata::{Metadata, MetadataAddress, MetadataProtocolId, ToMetadata},
    types::{Bytes, Result},
};

impl ToMetadata for EosOnIntEosTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        Ok(Metadata::new(
            &self.user_data,
            &MetadataAddress::new_from_eth_address(&self.token_sender, &self.origin_chain_id)?,
        ))
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Eos)
    }
}
