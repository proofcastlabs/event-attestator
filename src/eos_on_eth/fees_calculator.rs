use eos_chain::AccountName as EosAccountName;
use ethereum_types::{Address as EthAddress, U256};

use crate::{
    constants::FEE_BASIS_POINTS_DIVISOR,
    dictionaries::eos_eth::EosEthTokenDictionary,
    fees::fee_utils::sanity_check_basis_points_value,
    types::Result,
};

pub trait FeeCalculator {
    fn get_amount(&self) -> U256;

    fn get_eth_token_address(&self) -> EthAddress;

    fn get_eos_token_address(&self) -> Result<EosAccountName>;

    fn update_amount(&self, new_amount: U256, dictionary: &EosEthTokenDictionary) -> Result<Self>
    where
        Self: Sized;

    fn subtract_amount(&self, subtrahend: U256, dictionary: &EosEthTokenDictionary) -> Result<Self>
    where
        Self: Sized,
    {
        let amount = self.get_amount();
        if subtrahend >= amount {
            Err(format!("Cannot subtract amount, subtrahend of {} is too large!", subtrahend).into())
        } else {
            let new_amount = amount - subtrahend;
            debug!(
                "Subtracting {} from {} to get final amount of {}.",
                subtrahend, amount, new_amount
            );
            self.update_amount(new_amount, dictionary)
        }
    }

    fn calculate_fee(&self, fee_basis_points: u64) -> Result<U256> {
        sanity_check_basis_points_value(fee_basis_points).map(|_| {
            if fee_basis_points > 0 {
                let fee = (self.get_amount() * U256::from(fee_basis_points)) / U256::from(FEE_BASIS_POINTS_DIVISOR);
                info!(
                    "Calculated fee of {} using `fee_basis_points` of {}",
                    fee.as_u128(),
                    fee_basis_points
                );
                fee
            } else {
                debug!("Not calculating fee because `fee_basis_points` are zero!");
                U256::zero()
            }
        })
    }

    fn calculate_peg_out_fee_via_dictionary(&self, dictionary: &EosEthTokenDictionary) -> Result<(EthAddress, U256)> {
        let eth_token_address = self.get_eth_token_address();
        Ok((
            eth_token_address,
            self.calculate_fee(dictionary.get_eth_fee_basis_points(&eth_token_address)?)?,
        ))
    }

    fn calculate_peg_in_fee_via_dictionary(&self, dictionary: &EosEthTokenDictionary) -> Result<(EthAddress, U256)> {
        Ok((
            self.get_eth_token_address(),
            self.calculate_fee(dictionary.get_eos_fee_basis_points(&self.get_eos_token_address()?)?)?,
        ))
    }
}

pub trait FeesCalculator {
    fn get_fees(&self, dictionary: &EosEthTokenDictionary) -> Result<Vec<(EthAddress, U256)>>;

    fn subtract_fees(&self, dictionary: &EosEthTokenDictionary) -> Result<Self>
    where
        Self: Sized;
}
