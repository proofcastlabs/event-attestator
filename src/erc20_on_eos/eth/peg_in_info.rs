use ethereum_types::{
    U256,
    H256 as EthHash,
    Address as EthAddress,
};
use crate::{
    types::Result,
};
use derive_more::{
    Deref,
    Constructor,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Erc20OnEosPegInInfo {
    pub token_amount: U256,
    pub eos_address: String,
    pub token_sender: EthAddress,
    pub token_contract: EthAddress,
    pub originating_tx_hash: EthHash,
}

impl Erc20OnEosPegInInfo {
    pub fn new(
        token_amount: U256,
        token_sender: EthAddress,
        token_contract: EthAddress,
        eos_address: String,
        originating_tx_hash: EthHash
    ) -> Erc20OnEosPegInInfo {
        Erc20OnEosPegInInfo { token_amount, token_contract, eos_address, originating_tx_hash, token_sender }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Constructor, Deref)]
pub struct Erc20OnEosPegInInfos(pub Vec<Erc20OnEosPegInInfo>);

impl Erc20OnEosPegInInfos {
    pub fn sum(&self) -> U256 {
        self.0.iter().fold(U256::zero(), |acc, params| acc + params.token_amount)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}
