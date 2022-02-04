use derive_more::{Constructor, Deref, IntoIterator};
use ethereum_types::{Address as EthAddress, H256 as EthHash, U256};

use crate::{chains::eth::eth_chain_id::EthChainId, types::Bytes};

#[derive(Debug, Clone, PartialEq, Eq, Default, Constructor)]
pub struct EthOnIntEthTxInfo {
    pub native_token_amount: U256,
    pub token_sender: EthAddress,
    pub originating_tx_hash: EthHash,
    pub evm_token_address: EthAddress,
    pub eth_token_address: EthAddress,
    pub destination_address: EthAddress,
    pub user_data: Bytes,
    pub origin_chain_id: EthChainId,
    pub eth_vault_address: EthAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref, IntoIterator)]
pub struct EthOnIntEthTxInfos(pub Vec<EthOnIntEthTxInfo>);
