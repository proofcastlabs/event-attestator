use crate::{
    chains::btc::{btc_chain_id::BtcChainId, btc_utils::convert_str_to_btc_address_or_safe_address},
    metadata::{metadata_origin_address::MetadataOriginAddress, metadata_protocol_id::MetadataProtocolId, Metadata},
    types::{Byte, Bytes, Result},
};

pub trait ToMetadata {
    fn get_user_data(&self) -> Option<Bytes>;

    fn get_originating_tx_address(&self) -> String;

    fn maybe_to_metadata_bytes(
        &self,
        btc_chain_id: &BtcChainId,
        max_data_length: usize,
        destination_protocol_id: &MetadataProtocolId,
    ) -> Result<Option<Bytes>>
    where
        Self: Sized,
    {
        self.maybe_to_metadata(btc_chain_id, max_data_length)
            .and_then(|maybe_metadata| match maybe_metadata {
                Some(metadata) => Ok(Some(metadata.to_bytes_for_protocol(destination_protocol_id)?)),
                None => Ok(None),
            })
    }

    fn maybe_to_metadata(&self, btc_chain_id: &BtcChainId, max_data_length: usize) -> Result<Option<Metadata>>
    where
        Self: Sized,
    {
        info!("✔ Maybe getting metadata from user data...");
        match self.get_user_data() {
            Some(ref user_data) => {
                if user_data.len() > max_data_length {
                    info!(
                        "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes!",
                        max_data_length
                    );
                    Ok(None)
                } else {
                    self.to_metadata(user_data, btc_chain_id)
                }
            },
            None => {
                info!("✘ No user data to wrap into metadata ∴ skipping this step!");
                Ok(None)
            },
        }
    }

    fn to_metadata(&self, user_data: &[Byte], btc_chain_id: &BtcChainId) -> Result<Option<Metadata>> {
        info!("✔ Getting metadata from user data...");
        Ok(Some(Metadata::new(
            user_data,
            &MetadataOriginAddress::new_from_btc_address(
                &convert_str_to_btc_address_or_safe_address(&self.get_originating_tx_address())?,
                &btc_chain_id.to_metadata_chain_id(),
            )?,
        )))
    }
}
