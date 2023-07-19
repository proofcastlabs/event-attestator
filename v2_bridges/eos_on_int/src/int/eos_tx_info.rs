use common::types::{Byte, Bytes, Result};
use common_metadata::MetadataChainId;
use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Constructor, Deref, Serialize, Deserialize)]
pub struct EosOnIntEosTxInfos(pub Vec<EosOnIntEosTxInfo>);

impl EosOnIntEosTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        if self.is_empty() {
            Ok(vec![])
        } else {
            Ok(serde_json::to_vec(&self)?)
        }
    }

    pub fn from_bytes(bytes: &[Byte]) -> Result<Self> {
        if bytes.is_empty() {
            Ok(Self::default())
        } else {
            Ok(serde_json::from_slice(bytes)?)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EosOnIntEosTxInfo {
    pub user_data: Bytes,
    pub token_amount: U256,
    pub router_address: String,
    pub eos_asset_amount: String,
    pub token_sender: EthAddress,
    pub eos_token_address: String,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub int_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serde_empty_eos_tx_info_correctly() {
        let info = EosOnIntEosTxInfos::default();
        let result = info.to_bytes().unwrap();
        let expected_result: Bytes = vec![];
        assert_eq!(result, expected_result);
        let result_2 = EosOnIntEosTxInfos::from_bytes(&result).unwrap();
        assert_eq!(result_2, info);
    }
}
