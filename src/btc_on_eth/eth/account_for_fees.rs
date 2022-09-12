use crate::{
    btc_on_eth::eth::btc_tx_info::{BtcOnEthBtcTxInfo, BtcOnEthBtcTxInfos},
    chains::eth::eth_state::EthState,
    fees::{fee_constants::DISABLE_FEES, fee_database_utils::FeeDatabaseUtils},
    traits::DatabaseInterface,
    types::Result,
};

pub fn subtract_fees_from_btc_tx_infos(
    btc_tx_infos: &BtcOnEthBtcTxInfos,
    fee_basis_points: u64,
) -> Result<BtcOnEthBtcTxInfos> {
    btc_tx_infos.calculate_fees(fee_basis_points).and_then(|(fees, _)| {
        info!("ETH `BtcTxInfos` fees: {:?}", fees);
        Ok(BtcOnEthBtcTxInfos::new(
            fees.iter()
                .zip(btc_tx_infos.iter())
                .map(|(fee, btc_tx_info)| btc_tx_info.subtract_amount(*fee))
                .collect::<Result<Vec<BtcOnEthBtcTxInfo>>>()?,
        ))
    })
}

fn accrue_fees_from_btc_tx_infos<D: DatabaseInterface>(
    db: &D,
    btc_tx_infos: &BtcOnEthBtcTxInfos,
    fee_basis_points: u64,
) -> Result<()> {
    btc_tx_infos
        .calculate_fees(fee_basis_points)
        .and_then(|(_, total_fee)| {
            info!("ETH `BtcTxInfos` total fee: {}", total_fee);
            FeeDatabaseUtils::new_for_btc_on_eth().increment_accrued_fees(db, total_fee)
        })
}

fn account_for_fees_in_btc_tx_infos<D: DatabaseInterface>(
    db: &D,
    btc_tx_infos: &BtcOnEthBtcTxInfos,
    fee_basis_points: u64,
) -> Result<BtcOnEthBtcTxInfos> {
    if fee_basis_points == 0 {
        info!("✔ `BTC-on-ETH` peg-out fees are set to zero ∴ not taking any fees!");
        Ok(btc_tx_infos.clone())
    } else {
        info!("✔ Accounting for fees @ {} basis points...", fee_basis_points);
        accrue_fees_from_btc_tx_infos(db, btc_tx_infos, fee_basis_points)
            .and_then(|_| subtract_fees_from_btc_tx_infos(btc_tx_infos, fee_basis_points))
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EthState<D>) -> Result<EthState<D>> {
    info!("✔ Maybe accounting for fees...");
    if DISABLE_FEES {
        info!("✔ Taking fees is disabled ∴ not taking any fees!");
        Ok(state)
    } else if state.btc_on_eth_btc_tx_infos.is_empty() {
        info!("✔ Not redeem-info in state ∴ not taking any fees!");
        Ok(state)
    } else {
        account_for_fees_in_btc_tx_infos(
            state.db,
            &state.btc_on_eth_btc_tx_infos,
            FeeDatabaseUtils::new_for_btc_on_eth().get_peg_out_basis_points_from_db(state.db)?,
        )
        .and_then(|updated_btc_tx_infos| state.replace_btc_on_eth_btc_tx_infos(updated_btc_tx_infos))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{btc_on_eth::test_utils::get_sample_btc_on_eth_btc_tx_infos, test_utils::get_test_database};

    #[test]
    fn should_account_for_fees_in_btc_on_eth_btc_tx_infos() {
        let fee_basis_points = 25;
        let db = get_test_database();
        let accrued_fees_before = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        assert_eq!(accrued_fees_before, 0);
        let btc_tx_infos = get_sample_btc_on_eth_btc_tx_infos();
        let (_, total_fee) = btc_tx_infos.calculate_fees(fee_basis_points).unwrap();
        let expected_total_fee = 2777776;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = btc_tx_infos.sum();
        let resulting_infos = account_for_fees_in_btc_tx_infos(&db, &btc_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_infos.sum();
        let accrued_fees_after = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        let expected_peg_out_amount_after_1 = 123148148;
        let expected_peg_out_amount_after_2 = 985185186;
        assert_eq!(total_value_after + total_fee, total_value_before);
        assert_eq!(accrued_fees_after, total_fee);
        assert_eq!(resulting_infos[0].amount_in_satoshis, expected_peg_out_amount_after_1);
        assert_eq!(resulting_infos[1].amount_in_satoshis, expected_peg_out_amount_after_2);
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
        let btc_tx_infos = get_sample_btc_on_eth_btc_tx_infos();
        let (_, total_fee) = btc_tx_infos.calculate_fees(fee_basis_points).unwrap();
        let expected_total_fee = 0;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = btc_tx_infos.sum();
        let resulting_infos = account_for_fees_in_btc_tx_infos(&db, &btc_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_infos.sum();
        assert_eq!(total_value_before, total_value_after);
        let accrued_fees_after = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        assert_eq!(accrued_fees_after, 0);
    }

    #[test]
    fn should_account_for_fees_correctly_if_no_btc_tx_infos() {
        let fee_basis_points = 25;
        assert!(fee_basis_points > 0);
        let db = get_test_database();
        let accrued_fees_before = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        assert_eq!(accrued_fees_before, 0);
        let btc_tx_infos = BtcOnEthBtcTxInfos::new(vec![]);
        let (fees, total_fee) = btc_tx_infos.calculate_fees(fee_basis_points).unwrap();
        assert_eq!(fees, Vec::<u64>::new());
        let expected_total_fee = 0;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = btc_tx_infos.sum();
        let resulting_infos = account_for_fees_in_btc_tx_infos(&db, &btc_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_infos.sum();
        assert_eq!(total_value_before, total_value_after);
        let accrued_fees_after = FeeDatabaseUtils::new_for_btc_on_eth()
            .get_accrued_fees_from_db(&db)
            .unwrap();
        assert_eq!(accrued_fees_after, 0);
    }
}
