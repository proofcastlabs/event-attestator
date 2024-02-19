use derive_more::Constructor;

use crate::EthBlock;

#[derive(Clone, Constructor)]
pub struct Eip4844 {}

impl Eip4844 {
    pub fn is_active(&self, block: &EthBlock) -> Result<bool> {
        Ok(block.blob_gas_used.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::get_sample_eip4844_sepolia_submission_material;

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
