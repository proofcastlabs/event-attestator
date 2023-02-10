use common::EthChainId;
use ethereum_types::{Address, H256};

use crate::EthPrivateKey;

pub type EthHash = H256;
pub type EthAddress = Address;

#[derive(Debug)]
pub struct EthSigningParams {
    pub chain_id: EthChainId,
    pub gas_price: u64,
    pub eth_account_nonce: u64,
    pub eth_private_key: EthPrivateKey,
    pub smart_contract_address: EthAddress,
}

#[derive(Debug)]
pub struct AnySenderSigningParams {
    pub chain_id: EthChainId,
    pub any_sender_nonce: u64,
    pub eth_private_key: EthPrivateKey,
    pub public_eth_address: EthAddress,
    pub erc777_proxy_address: EthAddress,
}
