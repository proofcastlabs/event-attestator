use common::{traits::DatabaseInterface, types::Result};
use common_btc::BtcState;
use common_fees::{FeeDatabaseUtils, DISABLE_FEES};

use crate::btc::eos_tx_info::BtcOnEosEosTxInfos;

fn accrue_fees_from_eos_tx_infos<D: DatabaseInterface>(
    db: &D,
    eos_tx_infos: &BtcOnEosEosTxInfos,
    fee_basis_points: u64,
) -> Result<()> {
    eos_tx_infos
        .calculate_fees(fee_basis_points)
        .and_then(|(_, total_fee)| {
            info!("`BtcOnEosEosTxInfos` total fee: {}", total_fee);
            FeeDatabaseUtils::new_for_btc_on_eos().increment_accrued_fees(db, total_fee)
        })
}

fn account_for_fees_in_eos_tx_infos<D: DatabaseInterface>(
    db: &D,
    eos_tx_infos: &BtcOnEosEosTxInfos,
    fee_basis_points: u64,
) -> Result<BtcOnEosEosTxInfos> {
    if fee_basis_points == 0 {
        info!("✔ `BTC-on-EOS` peg-in fees are set to zero ∴ not taking any fees!");
        Ok(eos_tx_infos.clone())
    } else {
        info!("✔ Accounting for fees @ {} basis points...", fee_basis_points);
        accrue_fees_from_eos_tx_infos(db, eos_tx_infos, fee_basis_points)
            .and_then(|_| eos_tx_infos.subtract_fees(fee_basis_points))
    }
}

pub fn maybe_account_for_fees<D: DatabaseInterface>(state: BtcState<D>) -> Result<BtcState<D>> {
    info!("✔ Maybe accounting for fees...");
    if DISABLE_FEES {
        info!("✔ Taking fees is disabled ∴ not taking any fees!");
        Ok(state)
    } else if state.tx_infos.is_empty() {
        info!("✔ No `BtcOnEosEosTxInfos` in state ∴ not taking any fees!");
        Ok(state)
    } else {
        account_for_fees_in_eos_tx_infos(
            state.db,
            &BtcOnEosEosTxInfos::from_bytes(&state.tx_infos)?,
            FeeDatabaseUtils::new_for_btc_on_eos().get_peg_in_basis_points_from_db(state.db)?,
        )
        .and_then(|updated_tx_infos| updated_tx_infos.to_bytes())
        .map(|bytes| state.add_tx_infos(bytes))
    }
}

#[cfg(all(test, not(feature="ltc")))]
mod tests {
    use common::test_utils::get_test_database;
    use common_eos::convert_eos_asset_to_u64;

    use super::*;
    use crate::test_utils::get_sample_btc_on_eos_eos_tx_infos;

    #[test]
    fn should_account_for_fees_in_btc_on_eos_eos_tx_infos() {
        let fee_basis_points = 25;
        let db = get_test_database();
        let fee_db_utils = FeeDatabaseUtils::new_for_btc_on_eos();
        let accrued_fees_before = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_before, 0);
        let eos_tx_infos = get_sample_btc_on_eos_eos_tx_infos();
        let (_, total_fee) = eos_tx_infos.calculate_fees(fee_basis_points).unwrap();
        let expected_total_fee = 36;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = eos_tx_infos.sum();
        let resulting_params = account_for_fees_in_eos_tx_infos(&db, &eos_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        let accrued_fees_after = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        let expected_amount_after_1 = 4988;
        let expected_amount_after_2 = 4989;
        assert_eq!(total_value_after + total_fee, total_value_before);
        assert_eq!(accrued_fees_after, total_fee);
        assert_eq!(
            convert_eos_asset_to_u64(&resulting_params[0].amount).unwrap(),
            expected_amount_after_1
        );
        assert_eq!(
            convert_eos_asset_to_u64(&resulting_params[1].amount).unwrap(),
            expected_amount_after_2
        );
    }

    #[test]
    fn should_not_account_for_fees_in_btc_on_eos_eos_tx_infos_if_basis_points_are_zero() {
        let fee_basis_points = 0;
        assert_eq!(fee_basis_points, 0);
        let db = get_test_database();
        let fee_db_utils = FeeDatabaseUtils::new_for_btc_on_eos();
        let accrued_fees_before = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_before, 0);
        let eos_tx_infos = get_sample_btc_on_eos_eos_tx_infos();
        let (_, total_fee) = eos_tx_infos.calculate_fees(fee_basis_points).unwrap();
        let expected_total_fee = 0;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = eos_tx_infos.sum();
        let resulting_params = account_for_fees_in_eos_tx_infos(&db, &eos_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        assert_eq!(total_value_before, total_value_after);
        let accrued_fees_after = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_after, 0);
    }

    #[test]
    fn should_account_for_fees_correctly_in_btc_on_eos_eos_tx_infos_if_tx_infos_are_emtpy() {
        let fee_basis_points = 25;
        assert!(fee_basis_points > 0);
        let db = get_test_database();
        let fee_db_utils = FeeDatabaseUtils::new_for_btc_on_eos();
        let accrued_fees_before = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_before, 0);
        let eos_tx_infos = BtcOnEosEosTxInfos::new(vec![]);
        let (_, total_fee) = eos_tx_infos.calculate_fees(fee_basis_points).unwrap();
        let expected_total_fee = 0;
        assert_eq!(total_fee, expected_total_fee);
        let total_value_before = eos_tx_infos.sum();
        let resulting_params = account_for_fees_in_eos_tx_infos(&db, &eos_tx_infos, fee_basis_points).unwrap();
        let total_value_after = resulting_params.sum();
        assert_eq!(total_value_before, total_value_after);
        let accrued_fees_after = fee_db_utils.get_accrued_fees_from_db(&db).unwrap();
        assert_eq!(accrued_fees_after, 0);
    }
}
