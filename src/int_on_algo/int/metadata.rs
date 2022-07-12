use crate::{
    chains::{
        algo::{algo_constants::MAX_BYTES_FOR_ALGO_USER_DATA, algo_user_data::AlgoUserData},
        eth::eth_utils::convert_eth_address_to_string,
    },
    int_on_algo::int::algo_tx_info::IntOnAlgoAlgoTxInfo,
    metadata::{
        metadata_address::MetadataAddress,
        metadata_protocol_id::MetadataProtocolId,
        metadata_traits::ToMetadata,
        Metadata,
    },
    types::{Bytes, Result},
};

impl ToMetadata for IntOnAlgoAlgoTxInfo {
    fn to_metadata(&self) -> Result<Metadata> {
        info!("Getting metadata from `IntOnAlgoAlgoTxInfo`...");

        // NOTE: First we check if there is any user data...
        if self.user_data.is_empty() {
            return Err("Cannot wrap no `user_data` into metadata!".into());
        };

        // NOTE: Next we check if the user data provided can be decoded to `AlgoUserData`...
        let unchecked_user_data = match AlgoUserData::from_bytes(&self.user_data) {
            Err(_) => {
                info!("✔ Could not parse `AlgoUserData` from `user_data`, must not be msgpack encoded!");
                self.user_data.clone()
            },
            Ok(algo_user_data) => {
                info!("✔ Parsed `AlgoUserData` from `user_data`! Deriving `user_data` from that instead!");
                algo_user_data.to_user_data()
            },
        };

        // NOTE: Now we check that the length of the data is within our bounds...
        let user_data = if unchecked_user_data.len() > MAX_BYTES_FOR_ALGO_USER_DATA {
            info!(
                "✘ `user_data` redacted from `Metadata` ∵ it's > {} bytes",
                MAX_BYTES_FOR_ALGO_USER_DATA
            );
            vec![]
        } else {
            info!(
                "✔ User data to be wrapped in metadata: 0x{}",
                hex::encode(&unchecked_user_data)
            );
            unchecked_user_data
        };

        let destination_metadata_address = if self.destination_is_app() {
            MetadataAddress::new(&self.get_destination_app_id()?.to_string(), &self.destination_chain_id)?
        } else {
            MetadataAddress::new(&self.get_destination_address()?.to_string(), &self.destination_chain_id)?
        };

        let metadata = Metadata::new_v3(
            &user_data,
            &MetadataAddress::new(
                &convert_eth_address_to_string(&self.token_sender),
                &self.origin_chain_id,
            )?,
            &destination_metadata_address,
            None,
            None,
        );
        Ok(metadata)
    }

    fn to_metadata_bytes(&self) -> Result<Bytes> {
        self.to_metadata()?.to_bytes_for_protocol(&MetadataProtocolId::Algorand)
    }
}
