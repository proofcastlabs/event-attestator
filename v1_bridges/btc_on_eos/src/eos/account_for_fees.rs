use common::{
    fees::{fee_constants::DISABLE_FEES, fee_database_utils::FeeDatabaseUtils},
    state::EosState,
    traits::DatabaseInterface,
    types::Result,
};

use crate::eos::btc_tx_info::BtcOnEosBtcTxInfos;

fn accrue_fees_from_btc_tx_info<D: DatabaseInterface>(
    db: &D,
    btc_tx_info: &BtcOnEosBtcTxInfos,
    fee_basis_points: u64,
) -> Result<()> {
    let (_, total_fee) = btc_tx_info.calculate_fees(fee_basis_points);
    info!("`BtcOnEosBtcTxInfos` total fee: {}", total_fee);
    FeeDatabaseUtils::new_for_btc_on_eos().increment_accrued_fees(db, total_fee)
}

fn account_for_fees_in_btc_tx_infos<D: DatabaseInterface>(
    db: &D,
    btc_tx_infos: &BtcOnEosBtcTxInfos,
    fee_basis_points: u64,
) -> Result<BtcOnEosBtcTxInfos> {
    if fee_basis_points == 0 {
        info!("✔ `BTC-on-EOS` peg-in fees are set to zero ∴ not taking any fees!");
        Ok(btc_tx_infos.clone())
    } else {
        info!("✔ Accounting for fees @ {} basis points...", fee_basis_points);
        accrue_fees_from_btc_tx_info(db, btc_tx_infos, fee_basis_points)
            .and_then(|_| btc_tx_infos.subtract_fees(fee_basis_points))
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: EosState<D>) -> Result<EosState<D>> {
    info!("✔ Maybe accounting for fees...");
    if DISABLE_FEES {
        info!("✔ Taking fees is disabled ∴ not taking any fees!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ No `BtcOnEosBtcTxInfos` in state ∴ not taking any fees!");
        Ok(state)
    } else {
        BtcOnEosBtcTxInfos::from_bytes(&state.tx_infos)
            .and_then(|infos| {
                account_for_fees_in_btc_tx_infos(
                    state.db,
                    &infos,
                    FeeDatabaseUtils::new_for_btc_on_eos().get_peg_out_basis_points_from_db(state.db)?,
                )
            })
            .and_then(|updated_btc_tx_info| updated_btc_tx_info.to_bytes())
            .map(|bytes| state.add_tx_infos(bytes))
    }
}

#[cfg(test)]
mod tests {
    use common::test_utils::get_test_database;

    use super::*;
    use crate::test_utils::get_sample_btc_tx_infos;

    #[test]
    fn should_account_for_fees_in_btc_on_eos_btc_tx_infos() {
        let fee_basis_points = 25;
        let db = get_test_database();
        let fee_db_utils = FeeDatabaseUtils::new_for_btc_on_eos();
        let accrued_fees_before = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_before, 0);
        let btc_tx_infos = get_sample_btc_tx_infos();
        let (_, total_fee) = btc_tx_infos.calculate_fees(fee_basis_points);
        let expected_total_fee = 24;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = btc_tx_infos.sum();
        let resulting_params = account_for_fees_in_btc_tx_infos(&db, &btc_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        let accrued_fees_after = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        let expected_amount = 5099;
        assert_eq!(total_value_after + total_fee, total_value_before);
        assert_eq!(accrued_fees_after, total_fee);
        resulting_params
            .iter()
            .for_each(|params| assert_eq!(params.amount, expected_amount));
    }

    #[test]
    fn should_not_account_for_fees_in_btc_on_eos_btc_tx_info_if_basis_points_are_zero() {
        let fee_basis_points = 0;
        assert_eq!(fee_basis_points, 0);
        let db = get_test_database();
        let fee_db_utils = FeeDatabaseUtils::new_for_btc_on_eos();
        let accrued_fees_before = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_before, 0);
        let btc_tx_infos = get_sample_btc_tx_infos();
        let (_, total_fee) = btc_tx_infos.calculate_fees(fee_basis_points);
        let expected_total_fee = 0;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = btc_tx_infos.sum();
        let resulting_params = account_for_fees_in_btc_tx_infos(&db, &btc_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        assert_eq!(total_value_before, total_value_after);
        let accrued_fees_after = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_after, 0);
    }

    #[test]
    fn should_account_for_fees_correctly_in_btc_on_eos_btc_tx_info_if_btc_tx_infos_are_emtpy() {
        let fee_basis_points = 25;
        assert!(fee_basis_points > 0);
        let db = get_test_database();
        let fee_db_utils = FeeDatabaseUtils::new_for_btc_on_eos();
        let accrued_fees_before = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_before, 0);
        let btc_tx_infos = BtcOnEosBtcTxInfos::new(vec![]);
        let (_, total_fee) = btc_tx_infos.calculate_fees(fee_basis_points);
        let expected_total_fee = 0;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = btc_tx_infos.sum();
        let resulting_params = account_for_fees_in_btc_tx_infos(&db, &btc_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        assert_eq!(total_value_before, total_value_after);
        let accrued_fees_after = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_after, 0);
    }
}
