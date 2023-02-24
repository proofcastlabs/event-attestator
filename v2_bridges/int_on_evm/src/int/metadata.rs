use common::{
    safe_addresses::safely_convert_str_to_eth_address,
    types::{Bytes, Result},
};
use common_eth::MAX_BYTES_FOR_ETH_USER_DATA;
use common_metadata::{Metadata, MetadataAddress, MetadataProtocolId};

use crate::int::evm_tx_info::IntOnEvmEvmTxInfo;

impl IntOnEvmEvmTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        let user_data = if self.user_data.len() > MAX_BYTES_FOR_ETH_USER_DATA {
            // TODO Test for this case!
            info!(
                "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes",
                MAX_BYTES_FOR_ETH_USER_DATA
            );
            vec![]
        } else {
            self.user_data.clone()
        };
        Ok(Metadata::new_v2(
            &user_data,
            &MetadataAddress::new_from_eth_address(&self.token_sender, &self.origin_chain_id)?,
            &MetadataAddress::new_from_eth_address(
                &safely_convert_str_to_eth_address(&self.destination_address),
                &self.destination_chain_id,
            )?,
            None,
            None,
        ))
    }

    pub fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Ethereum)
    }
}
