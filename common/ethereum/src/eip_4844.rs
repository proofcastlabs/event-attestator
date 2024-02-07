use common::types::Result;
use common_chain_ids::EthChainId;
use derive_more::Constructor;
use ethereum_types::U256;

#[derive(Clone, Constructor)]
pub struct Eip4844 {}

impl Eip4844 {
    pub fn get_activation_block_number(&self, eth_chain_id: &EthChainId) -> Result<U256> {
        match eth_chain_id {
            EthChainId::Goerli => Ok(U256::from(10_456_115)),
            EthChainId::Sepolia => Ok(U256::from(5_207_710)),
            _ => Err(format!("{} does not have an `EIP4844` activation block number! ", eth_chain_id).into()),
        }
    }

    pub fn is_active(&self, eth_chain_id: &EthChainId, block_number: U256) -> Result<bool> {
        match eth_chain_id {
            EthChainId::Sepolia | EthChainId::Goerli => {
                Ok(block_number >= self.get_activation_block_number(eth_chain_id)?)
            },
            _ => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_eip4844_get_activation_block() {
        let eip4844 = Eip4844::new();
        let chain_id = EthChainId::Sepolia;
        let result = eip4844.get_activation_block_number(&chain_id).unwrap();
        let expected_result = U256::from(5_207_710);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn eip4844_should_be_active() {
        let block_number = U256::from(13_000_000);
        let eip_4844 = Eip4844::new();
        let chain_id = EthChainId::Sepolia;
        let result = eip_4844.is_active(&chain_id, block_number).unwrap();
        assert!(result);
    }

    #[test]
    fn eip_4844_should_not_be_active() {
        let block_number = U256::from(5_000_000);
        let eip_4844 = Eip4844::new();
        let chain_id = EthChainId::Sepolia;
        let result = eip_4844.is_active(&chain_id, block_number).unwrap();
        assert!(!result);
    }
}
