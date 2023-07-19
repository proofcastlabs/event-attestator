use common::{
    address::Address,
    types::{Byte, Bytes, Result},
};
use common_eos::{GlobalSequence, GlobalSequences};
use common_metadata::MetadataChainId;
use common_safe_addresses::SAFE_ETH_ADDRESS_STR;
use derive_more::{Constructor, Deref};
use eos_chain::{AccountName as EosAccountName, Checksum256};
use ethereum_types::{Address as EthAddress, U256};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Default, Deref, Constructor, Serialize, Deserialize)]
pub struct IntOnEosIntTxInfos(pub Vec<IntOnEosIntTxInfo>);

impl IntOnEosIntTxInfos {
    pub fn to_bytes(&self) -> Result<Bytes> {
        if self.is_empty() {
            Ok(vec![])
        } else {
            Ok(serde_json::to_vec(self)?)
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct IntOnEosIntTxInfo {
    pub amount: U256,
    pub user_data: Bytes,
    pub eos_tx_amount: String,
    pub eos_token_address: String,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub int_vault_address: EthAddress,
    pub int_token_address: EthAddress,
    pub origin_address: EosAccountName,
    pub originating_tx_id: Checksum256,
    pub global_sequence: GlobalSequence,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}

impl_tx_info_trait!(
    IntOnEosIntTxInfo,
    int_vault_address,
    router_address,
    int_token_address,
    destination_address,
    Address::Eth,
    SAFE_ETH_ADDRESS_STR
);

impl IntOnEosIntTxInfos {
    pub fn get_global_sequences(&self) -> GlobalSequences {
        GlobalSequences::new(
            self.iter()
                .map(|info| info.global_sequence)
                .collect::<Vec<GlobalSequence>>(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serde_empty_int_tx_info_correctly() {
        let info = IntOnEosIntTxInfos::default();
        let result = info.to_bytes().unwrap();
        let expected_result: Bytes = vec![];
        assert_eq!(result, expected_result);
        let result_2 = IntOnEosIntTxInfos::from_bytes(&result).unwrap();
        assert_eq!(result_2, info);
    }
}
