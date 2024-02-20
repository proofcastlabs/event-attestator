use std::convert::TryFrom;

use common::{types::Bytes, AppError};
use common_metadata::{MetadataChainId, METADATA_CHAIN_ID_NUMBER_OF_BYTES};
use derive_getters::Getters;
use derive_more::Constructor;
use ethabi::{decode as eth_abi_decode, ParamType as EthAbiParamType, Token as EthAbiToken};
use serde::{Deserialize, Serialize};

use crate::{EthLog, EthLogExt, EthReceipt};

crate::make_topics!(
    PTOKENS_ROUTER_METADATA_EVENT_TOPIC => "41954c3bf6e497b17fc12f429900878df830619bbcccb5f61aedc91e6ccc9e64",
);

#[derive(Clone, Debug, Default, Eq, PartialEq, Constructor, Deserialize, Getters, Serialize)]
pub struct PTokensRouterMetadataEvent {
    user_data: Bytes,
    origin_chain_id: MetadataChainId,
    origin_address: String,
    destination_chain_id: MetadataChainId,
    destination_address: String,
}

impl TryFrom<&EthLog> for PTokensRouterMetadataEvent {
    type Error = AppError;

    fn try_from(log: &EthLog) -> Result<Self, Self::Error> {
        info!("decoding `PTokensRouterMetadataEvent` from log...");

        fn get_err_msg(field: &str) -> String {
            format!("Error getting `{}` from `PTokensRouterMetadataEvent`!", field)
        }

        let tokens = eth_abi_decode(
            &[
                EthAbiParamType::Bytes,
                EthAbiParamType::FixedBytes(METADATA_CHAIN_ID_NUMBER_OF_BYTES),
                EthAbiParamType::String,
                EthAbiParamType::FixedBytes(METADATA_CHAIN_ID_NUMBER_OF_BYTES),
                EthAbiParamType::String,
            ],
            &log.get_data(),
        )?;

        Ok(Self {
            user_data: match tokens[0] {
                EthAbiToken::Bytes(ref bytes) => Ok(bytes.to_vec()),
                _ => Err(get_err_msg("user_data")),
            }?,
            origin_chain_id: match tokens[1] {
                EthAbiToken::FixedBytes(ref bytes) => Ok(MetadataChainId::from_bytes(bytes)?),
                _ => Err(get_err_msg("origin_chain_id")),
            }?,
            origin_address: match tokens[2] {
                EthAbiToken::String(ref string) => Ok(string.to_string()),
                _ => Err(get_err_msg("origin_address")),
            }?,
            destination_chain_id: match tokens[3] {
                EthAbiToken::FixedBytes(ref bytes) => Ok(MetadataChainId::from_bytes(bytes)?),
                _ => Err(get_err_msg("destination_chain_id")),
            }?,
            destination_address: match tokens[4] {
                EthAbiToken::String(ref string) => Ok(string.to_string()),
                _ => Err(get_err_msg("destination_address")),
            }?,
        })
    }
}

impl TryFrom<&EthReceipt> for PTokensRouterMetadataEvent {
    type Error = AppError;

    fn try_from(receipt: &EthReceipt) -> Result<Self, Self::Error> {
        info!("decoding `PTokensRouterMetadataEvent` from receipt...");

        match receipt
            .logs
            .iter()
            .find(|log| log.contains_topic(&PTOKENS_ROUTER_METADATA_EVENT_TOPIC))
        {
            Some(log) => Self::try_from(log),
            None => Err("no matching log found".to_string())?,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use crate::eth_contracts::test_utils::get_ptokens_router_metadata_event_receipt;

    #[test]
    fn should_decode_metadata_event_from_receipt() {
        let expected_event = PTokensRouterMetadataEvent {
            user_data: vec![192, 255, 238],
            origin_chain_id: MetadataChainId::from_str("0x00f34368").unwrap(),
            origin_address: "0xfEDFe2616EB3661CB8FEd2782F5F0cC91D59DCaC".to_string(),
            destination_chain_id: MetadataChainId::from_str("0xffffffff").unwrap(),
            destination_address: "0xedB86cd455ef3ca43f0e227e00469C3bDFA40628".to_string(),
        };

        let receipt = get_ptokens_router_metadata_event_receipt().unwrap();
        let event = PTokensRouterMetadataEvent::try_from(&receipt).unwrap();

        assert_eq!(expected_event, event);
    }
}
