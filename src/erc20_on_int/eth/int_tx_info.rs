use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{metadata::metadata_chain_id::MetadataChainId, types::Bytes};

#[derive(Debug, Clone, Default, PartialEq, Eq, Constructor)]
pub struct EthOnIntIntTxInfo {
    pub native_token_amount: U256,
    pub token_sender: EthAddress,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_token_address: EthAddress,
    pub destination_address: EthAddress,
    pub user_data: Bytes,
    pub origin_chain_id: MetadataChainId,
    pub router_address: EthAddress,
    pub destination_chain_id: MetadataChainId,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct EthOnIntIntTxInfos(pub Vec<EthOnIntIntTxInfo>);
