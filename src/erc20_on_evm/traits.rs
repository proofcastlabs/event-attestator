use ethereum_types::{Address as EthAddress, U256};

use crate::{dictionaries::eth_evm::EthEvmTokenDictionary, types::Result};

pub trait FeeCalculator {
    fn get_amount(&self) -> U256;

    fn get_token_address(&self) -> EthAddress;

    fn subtract_amount(&self, subtrahend: U256) -> Self;

    fn calculate_fee(&self, fee_basis_points: u64) -> U256 {
        (self.get_amount() * U256::from(fee_basis_points)) / U256::from(10_000)
    }

    fn calculate_fee_via_dictionary(&self, dictionary: &EthEvmTokenDictionary) -> Result<(EthAddress, U256)> {
        let token_address = self.get_token_address();
        Ok((
            token_address,
            self.calculate_fee(dictionary.get_fee_basis_points(&token_address)?),
        ))
    }
}

pub trait FeesCalculator {
    fn get_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Vec<(EthAddress, U256)>>;

    fn subtract_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Self>
    where
        Self: Sized;
}
