use common::{traits::DatabaseInterface, types::Result};
use common_btc::BtcState;
use common_fees::{sanity_check_basis_points_value, FeeDatabaseUtils, DISABLE_FEES, FEE_BASIS_POINTS_DIVISOR};
use ethereum_types::U256;

use crate::{
    btc::eth_tx_info::{BtcOnEthEthTxInfo, BtcOnEthEthTxInfos},
    utils::{convert_satoshis_to_wei, convert_wei_to_satoshis},
};

impl BtcOnEthEthTxInfos {
    #[cfg(test)]
    fn sum(&self) -> U256 {
        self.iter().fold(U256::zero(), |a, params| a + params.amount)
    }

    fn calculate_fees(&self, basis_points: u64) -> Result<(Vec<u64>, u64)> {
        sanity_check_basis_points_value(basis_points).map(|_| {
            info!("✔ Calculating fees in `BtcOnEthEthTxInfos`...");
            let fees = self
                .iter()
                .map(|eth_tx_infos| eth_tx_infos.calculate_fee(basis_points))
                .collect::<Vec<u64>>();
            let total_fee = fees.iter().sum();
            info!("✔      Fees: {:?}", fees);
            info!("✔ Total fee: {:?}", fees);
            (fees, total_fee)
        })
    }
}

impl BtcOnEthEthTxInfo {
    fn to_satoshi_amount(&self) -> u64 {
        convert_wei_to_satoshis(self.amount)
    }

    fn calculate_fee(&self, basis_points: u64) -> u64 {
        (self.to_satoshi_amount() * basis_points) / FEE_BASIS_POINTS_DIVISOR
    }

    fn update_amount(&self, new_amount: U256) -> Self {
        let mut new_self = self.clone();
        new_self.amount = new_amount;
        new_self
    }

    fn subtract_satoshi_amount(&self, subtrahend: u64) -> Result<Self> {
        let self_amount_in_satoshis = self.to_satoshi_amount();
        if subtrahend > self_amount_in_satoshis {
            Err("Cannot subtract amount from `BtcOnEthEthTxInfo`: subtrahend too large!".into())
        } else {
            let amount_minus_fee = self_amount_in_satoshis - subtrahend;
            debug!(
                "Subtracted amount of {} from current eth tx infos amount of {} to get final amount of {}",
                subtrahend, self_amount_in_satoshis, amount_minus_fee
            );
            Ok(self.update_amount(convert_satoshis_to_wei(amount_minus_fee)))
        }
    }
}

pub fn subtract_fees_from_eth_tx_infos(
    eth_tx_infos: &BtcOnEthEthTxInfos,
    fee_basis_points: u64,
) -> Result<BtcOnEthEthTxInfos> {
    eth_tx_infos.calculate_fees(fee_basis_points).and_then(|(fees, _)| {
        info!("BTC `MintingParam` fees: {:?}", fees);
        Ok(BtcOnEthEthTxInfos::new(
            fees.iter()
                .zip(eth_tx_infos.iter())
                .map(|(fee, eth_tx_info)| eth_tx_info.subtract_satoshi_amount(*fee))
                .collect::<Result<Vec<BtcOnEthEthTxInfo>>>()?,
        ))
    })
}

fn accrue_fees_from_eth_tx_infos<D: DatabaseInterface>(
    db: &D,
    eth_tx_infos: &BtcOnEthEthTxInfos,
    fee_basis_points: u64,
) -> Result<()> {
    eth_tx_infos
        .calculate_fees(fee_basis_points)
        .and_then(|(_, total_fee)| {
            info!("BTC `EthTxInfos` total fee: {}", total_fee);
            FeeDatabaseUtils::new_for_btc_on_eth().increment_accrued_fees(db, total_fee)
        })
}

fn account_for_fees_in_eth_tx_infos<D: DatabaseInterface>(
    db: &D,
    eth_tx_infos: &BtcOnEthEthTxInfos,
    fee_basis_points: u64,
) -> Result<BtcOnEthEthTxInfos> {
    if fee_basis_points == 0 {
        info!("✔ `BTC-on-ETH` peg-in fees are set to zero ∴ not taking any fees!");
        Ok(eth_tx_infos.clone())
    } else {
        info!("✔ Accounting for fees @ {} basis points...", fee_basis_points);
        accrue_fees_from_eth_tx_infos(db, eth_tx_infos, fee_basis_points)
            .and_then(|_| subtract_fees_from_eth_tx_infos(eth_tx_infos, fee_basis_points))
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Maybe accounting for fees...");
    if DISABLE_FEES {
        info!("✔ Taking fees is disabled ∴ not taking any fees!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ Not `BtcOnEthEthTxInfos` in state ∴ not taking any fees!");
        Ok(state)
    } else {
        BtcOnEthEthTxInfos::from_bytes(&state.tx_infos)
            .and_then(|tx_infos| {
                account_for_fees_in_eth_tx_infos(
                    state.db,
                    &tx_infos,
                    FeeDatabaseUtils::new_for_btc_on_eth().get_peg_in_basis_points_from_db(state.db)?,
                )
            })
            .and_then(|updated_eth_tx_infos| updated_eth_tx_infos.to_bytes())
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

#[cfg(test)]
mod tests {
    use common::{errors::AppError, test_utils::get_test_database};
    use common_fees::FeeDatabaseUtils;

    use super::*;
    use crate::{test_utils::get_sample_eth_tx_infos, utils::convert_satoshis_to_wei};

    #[test]
    fn should_account_for_fees_in_btc_on_eth_eth_tx_infos() {
        let fee_basis_points = 25;
        let db = get_test_database();
        let accrued_fees_before = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        assert_eq!(accrued_fees_before, 0);
        let eth_tx_infos = get_sample_eth_tx_infos();
        let (_, total_fee) = eth_tx_infos.calculate_fees(fee_basis_points).unwrap();
        let expected_total_fee = 36;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = eth_tx_infos.sum();
        let resulting_params = account_for_fees_in_eth_tx_infos(&db, &eth_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        let accrued_fees_after = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        let expected_amount_after_1 = convert_satoshis_to_wei(4988);
        let expected_amount_after_2 = convert_satoshis_to_wei(4989);
        let expected_amount_after_3 = convert_satoshis_to_wei(4987);
        assert_eq!(
            total_value_after + convert_satoshis_to_wei(total_fee),
            total_value_before
        );
        assert_eq!(accrued_fees_after, total_fee);
        assert_eq!(resulting_params[0].amount, expected_amount_after_1);
        assert_eq!(resulting_params[1].amount, expected_amount_after_2);
        assert_eq!(resulting_params[2].amount, expected_amount_after_3);
    }

    #[test]
    fn should_not_account_for_fees_if_basis_points_are_zero() {
        let fee_basis_points = 0;
        assert_eq!(fee_basis_points, 0);
        let db = get_test_database();
        let accrued_fees_before = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        assert_eq!(accrued_fees_before, 0);
        let eth_tx_infos = get_sample_eth_tx_infos();
        let (_, total_fee) = eth_tx_infos.calculate_fees(fee_basis_points).unwrap();
        let expected_total_fee = 0;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = eth_tx_infos.sum();
        let resulting_params = account_for_fees_in_eth_tx_infos(&db, &eth_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        assert_eq!(total_value_before, total_value_after);
        let accrued_fees_after = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        assert_eq!(accrued_fees_after, 0);
    }

    #[test]
    fn should_account_for_fees_correctly_if_eth_tx_infos_are_emtpy() {
        let fee_basis_points = 25;
        assert!(fee_basis_points > 0);
        let db = get_test_database();
        let accrued_fees_before = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        assert_eq!(accrued_fees_before, 0);
        let eth_tx_infos = BtcOnEthEthTxInfos::new(vec![]);
        let (_, total_fee) = eth_tx_infos.calculate_fees(fee_basis_points).unwrap();
        let expected_total_fee = 0;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = eth_tx_infos.sum();
        let resulting_params = account_for_fees_in_eth_tx_infos(&db, &eth_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        assert_eq!(total_value_before, total_value_after);
        let accrued_fees_after = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        assert_eq!(accrued_fees_after, 0);
    }

    #[test]
    fn should_calculate_fee() {
        let params = get_sample_eth_tx_infos()[0].clone();
        let basis_points = 25;
        let expected_result = 12;
        let result = params.calculate_fee(basis_points);
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_calculate_fees() {
        let basis_points = 25;
        let params = get_sample_eth_tx_infos();
        let (fees, total_fee) = params.calculate_fees(basis_points).unwrap();
        let expected_total_fee = 36;
        let expected_fees = vec![12, 12, 12];
        assert_eq!(total_fee, expected_total_fee);
        assert_eq!(fees, expected_fees);
    }

    #[test]
    fn should_get_amount_in_satoshi() {
        let params = get_sample_eth_tx_infos()[0].clone();
        let result = params.to_satoshi_amount();
        let expected_result = 5000;
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_subtract_satoshi_amount() {
        let params = get_sample_eth_tx_infos()[0].clone();
        let subtracted_params = params.subtract_satoshi_amount(1).unwrap();
        let expected_result = 4999;
        let result = subtracted_params.to_satoshi_amount();
        assert_eq!(result, expected_result);
    }

    #[test]
    fn should_error_subtracting_amount_if_subtrahend_is_too_large() {
        let params = get_sample_eth_tx_infos()[0].clone();
        let subtrahend = (params.amount + 1).as_u64();
        let expected_error = "Cannot subtract amount from `BtcOnEthEthTxInfo`: subtrahend too large!";
        match params.subtract_satoshi_amount(subtrahend) {
            Ok(_) => panic!("Should not have succeeded!"),
            Err(AppError::Custom(error)) => assert_eq!(error, expected_error),
            Err(_) => panic!("Wrong error received!"),
        }
    }
}
