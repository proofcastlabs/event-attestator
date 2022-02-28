use derive_more::{Constructor, Deref};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{metadata::metadata_chain_id::MetadataChainId, types::Bytes};

#[derive(Debug, Clone, Default, PartialEq, Eq, Constructor)]
pub struct Erc20OnIntIntTxInfo {
    pub user_data: Bytes,
    pub token_sender: EthAddress,
    pub native_token_amount: U256,
    pub evm_token_address: String,
    pub router_address: EthAddress,
    pub destination_address: String,
    pub originating_tx_hash: EthHash,
    pub eth_token_address: EthAddress,
    pub origin_chain_id: MetadataChainId,
    pub destination_chain_id: MetadataChainId,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct Erc20OnIntIntTxInfos(pub Vec<Erc20OnIntIntTxInfo>);
