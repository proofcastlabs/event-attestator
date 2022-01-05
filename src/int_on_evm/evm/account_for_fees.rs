use ethereum_types::{Address as EthAddress, U256};

use crate::{
    chains::eth::eth_state::EthState,
    dictionaries::eth_evm::EthEvmTokenDictionary,
    fees::fee_constants::DISABLE_FEES,
    int_on_evm::{
        evm::int_tx_info::{IntOnEvmIntTxInfo, IntOnEvmIntTxInfos},
        fees_calculator::{FeeCalculator, FeesCalculator},
    },
    traits::DatabaseInterface,
    types::Result,
};

impl FeeCalculator for IntOnEvmIntTxInfo {
    fn get_token_address(&self) -> EthAddress {
        debug!(
            "Getting token address in `IntOnEvmIntTxInfo` of {}",
            self.evm_token_address
        );
        self.evm_token_address
    }

    fn get_amount(&self) -> U256 {
        debug!(
            "Getting token amount in `IntOnEvmIntTxInfo` of {}",
            self.native_token_amount
        );
        self.native_token_amount
    }

    fn subtract_amount(&self, subtrahend: U256) -> Result<Self> {
        if subtrahend >= self.native_token_amount {
            Err("Cannot subtract amount from `IntOnEvmIntTxInfo`: subtrahend too large!".into())
        } else {
            let new_amount = self.native_token_amount - subtrahend;
            debug!(
                "Subtracting {} from {} to get final amount of {} in `IntOnEvmIntTxInfo`!",
                subtrahend, self.native_token_amount, new_amount
            );
            Ok(self.update_amount(new_amount))
        }
    }
}

impl IntOnEvmIntTxInfo {
    fn update_amount(&self, new_amount: U256) -> Self {
        let mut new_self = self.clone();
        new_self.native_token_amount = new_amount;
        new_self
    }
}

impl FeesCalculator for IntOnEvmIntTxInfos {
    fn get_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Vec<(EthAddress, U256)>> {
        debug!("Calculating fees in `IntOnEvmIntTxInfo`...");
        self.iter()
            .map(|info| info.calculate_fee_via_dictionary(dictionary))
            .collect()
    }

    fn subtract_fees(&self, dictionary: &EthEvmTokenDictionary) -> Result<Self> {
        self.get_fees(dictionary).and_then(|fee_tuples| {
            Ok(Self::new(
                self.iter()
                    .zip(fee_tuples.iter())
                    .map(|(info, (_, fee))| {
                        if *fee == U256::zero() {
                            debug!("Not subtracting fee because `fee` is 0!");
                            Ok(info.clone())
                        } else {
                            info.subtract_amount(*fee)
                        }
                    })
                    .collect::<Result<Vec<IntOnEvmIntTxInfo>>>()?,
            ))
        })
    }
}

pub fn update_accrued_fees_in_dictionary_and_return_state<D: DatabaseInterface>(
    state: EthState<D>,
) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `IntOnEvmIntTxInfos`!");
        Ok(state)
    } else if state.int_on_evm_int_tx_infos.is_empty() {
        info!("✔ Not `IntOnEvmIntTxInfos` in state ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accruing fees during EVM block submission...");
        EthEvmTokenDictionary::get_from_db(state.db)
            .and_then(|ref dictionary| {
                dictionary.increment_accrued_fees_and_save_in_db(
                    state.db,
                    state.int_on_evm_int_tx_infos.get_fees(dictionary)?,
                )
            })
            .and(Ok(state))
    }
}

pub fn account_for_fees_in_eth_tx_infos_in_state<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    if DISABLE_FEES {
        info!("✔ Fees are disabled ∴ not accounting for any in `IntOnEvmIntTxInfos`!");
        Ok(state)
    } else if state.int_on_evm_int_tx_infos.is_empty() {
        info!("✔ Not `IntOnEvmIntTxInfos` in state ∴ not taking any fees!");
        Ok(state)
    } else {
        info!("✔ Accounting for fees in `IntOnEvmIntTxInfos` during EVM block submission...");
        EthEvmTokenDictionary::get_from_db(state.db).and_then(|ref dictionary| {
            let tx_infos = state.int_on_evm_int_tx_infos.clone();
            state.replace_int_on_evm_int_tx_infos(tx_infos.subtract_fees(dictionary)?)
        })
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Accounting for fees in `IntOnEvmIntTxInfos` during EVM block submission...");
    update_accrued_fees_in_dictionary_and_return_state(state).and_then(account_for_fees_in_eth_tx_infos_in_state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int_on_evm::test_utils::{
        get_sample_peg_out_submission_material,
        get_sample_router_address,
        get_sample_token_dictionary,
    };

    fn get_sample_tx_infos() -> IntOnEvmIntTxInfos {
        let dictionary = get_sample_token_dictionary();
        let material = get_sample_peg_out_submission_material();
        let router_address = get_sample_router_address();
        IntOnEvmIntTxInfos::from_submission_material(&material, &dictionary, &router_address).unwrap()
    }

    fn get_sample_tx_info() -> IntOnEvmIntTxInfo {
        get_sample_tx_infos()[0].clone()
    }

    #[test]
    fn should_calculate_eth_on_evm_eth_tx_info_fee() {
        let fee_basis_points = 25;
        let info = get_sample_tx_info();
        let result = info.calculate_fee(fee_basis_points);
        let expected_result = U256::from_dec_str("1").unwrap();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_amount_from_eth_on_evm_eth_tx_info() {
        let info = get_sample_tx_info();
        let subtrahend = U256::from(337);
        let result = info.subtract_amount(subtrahend).unwrap();
        let expected_native_token_amount = U256::from(329);
        assert_eq!(result.native_token_amount, expected_native_token_amount)
    }

    #[test]
    fn should_fail_to_subtract_too_large_amount_from_eth_on_evm_eth_tx_info() {
        let info = get_sample_tx_info();
        let subtrahend = U256::from(info.native_token_amount + 1);
        let result = info.subtract_amount(subtrahend);
        assert!(result.is_err());
    }
}
