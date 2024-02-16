use common::types::Result;
use common_chain_ids::EthChainId;
use derive_more::Constructor;
use ethereum_types::U256;

use crate::EthBlock;

#[derive(Clone, Constructor)]
pub struct Eip4844 {}

impl Eip4844 {
    #[allow(dead_code)]
    pub fn get_activation_block_number(&self, eth_chain_id: &EthChainId) -> Result<U256> {
        match eth_chain_id {
            EthChainId::Goerli => Ok(U256::from(10_456_115)),
            EthChainId::Sepolia => Ok(U256::from(5_207_710)),
            _ => Err(format!("{} does not have an `EIP4844` activation block number! ", eth_chain_id).into()),
        }
    }

    pub fn is_active(&self, block: &EthBlock) -> Result<bool> {
        Ok(block.blob_gas_used.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_eip4844_sepolia_submission_material;

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
        let eip_4844 = Eip4844::new();
        let block = get_sample_eip4844_sepolia_submission_material();
        let result = eip_4844.is_active(&block.block.unwrap()).unwrap();
        assert!(result);
    }

    #[test]
    fn eip_4844_should_not_be_active() {
        let eip_4844 = Eip4844::new();

        let block = get_sample_eip4844_sepolia_submission_material();
        let mut block = block.block.unwrap();
        block.blob_gas_used = None;
        let result = eip_4844.is_active(&block).unwrap();
        assert!(!result);
    }
}
